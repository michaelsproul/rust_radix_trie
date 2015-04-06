extern crate nibble_vec;

pub use nibble_vec::NibbleVec;
pub use keys::TrieKey;
use keys::{match_keys, check_keys, KeyMatch};
use DeleteAction::*;

mod keys;
#[cfg(test)]
mod test;

const BRANCH_FACTOR: usize = 16;

macro_rules! no_children {
    () => ([
        None, None, None, None,
        None, None, None, None,
        None, None, None, None,
        None, None, None, None
    ])
}

/// Tries allow collections of string-like keys to be efficiently stored and queried.
///
/// Any keys which share a common *prefix* are stored below a single copy of that prefix.
/// This saves space, and also allows the longest prefix of any given key to be found.
///
/// You can read more about Radix Tries on [Wikipedia][radix-wiki].
///
/// [radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
#[derive(Debug)]
pub struct Trie<K, V> {
    root: TrieNode<K, V>,
    length: usize
}

#[derive(Debug)]
struct TrieNode<K, V> {
    /// Key fragments/bits associated with this node, such that joining the keys from all
    /// parent nodes is equal to the bit-encoding of this node's key.
    key: NibbleVec,

    /// The key and value stored at this node.
    key_value: Option<KeyValue<K, V>>,

    /// The children of this node stored such that the first nibble of each child key
    /// dictates the child's bucket.
    children: [Option<Box<TrieNode<K, V>>>; BRANCH_FACTOR],

    /// The number of children which are Some rather than None.
    child_count: usize
}

#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V
}

#[derive(Debug)]
enum DeleteAction<K, V> {
    Replace(Box<TrieNode<K, V>>),
    Delete,
    DoNothing
}

impl<K, V> Trie<K, V> where K: TrieKey {
    /// Create an empty Trie with no data.
    pub fn new() -> Trie<K, V> {
        Trie {
            root: TrieNode {
                key: NibbleVec::new(),
                key_value: None,
                children: no_children![],
                child_count: 0
            },
            length: 0
        }
    }

    /// Fetch the number of key-value pairs stored in the Trie.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Determine if the Trie contains 0 key-value pairs.
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Fetch a reference to the given key's corresponding value (if any).
    pub fn get(&self, key: &K) -> Option<&V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        get(&self.root, key, key_fragments)
    }

    /// Fetch a mutable reference to the given key's corresponding value (if any).
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        get_mut(&mut self.root, key, key_fragments)
    }

    /// Insert the given key-value pair, returning any previous value associated with the key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        let result = self.root.insert(key, value, key_fragments);

        if result.is_none() {
            self.length += 1;
        }

        result
    }

    /// Remove and return the value associated with the given key.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());

        // Use the TrieNode recursive removal function but ignore its delete action.
        // The root can't be replaced or deleted.
        let (result, _) = self.root.remove_recursive(key, key_fragments);

        if result.is_some() {
            self.length -= 1;
        }

        result
    }

    /// Check that the Trie invariants are satisfied - you shouldn't ever have to call this!
    /// Quite slow!
    #[doc(hidden)]
    pub fn check_integrity(&self) -> bool {
        match self.root.check_integrity(&NibbleVec::new()) {
            (false, _) => false,
            (true, size) => size == self.length
        }
    }
}

/// Identity macro to allow expansion of the "mutability" token tree.
macro_rules! id {
    ($e:item) => { $e }
}

/// Macro to parametrise over mutability in get methods.
macro_rules! get_function {
    (
        name: $name:ident,
        mutability: $($mut_:tt)*
    ) => {
    id!(fn $name<'a, K, V>(
            trie: &'a $($mut_)* TrieNode<K, V>,
            key: &K,
            mut key_fragments: NibbleVec)
            -> Option<&'a $($mut_)* V> where K: TrieKey
        {
            // Handle retrieval at the root.
            if key_fragments.len() == 0 {
                return match trie.key_value {
                    Some(ref $($mut_)* kv) => Some(& $($mut_)* kv.value),
                    None => None
                };
            }

            let bucket = key_fragments.get(0) as usize;

            match trie.children[bucket] {
                None => None,
                Some(ref $($mut_)* existing_child) => {
                    match match_keys(&key_fragments, &existing_child.key) {
                        KeyMatch::Full => {
                            match existing_child.key_value {
                                Some(ref $($mut_)* kv) => {
                                    check_keys(&kv.key, key);
                                    Some(& $($mut_)* kv.value)
                                },
                                None => None
                            }
                        },

                        KeyMatch::Partial(idx) => {
                            let new_key_fragments = key_fragments.split(idx);

                            $name(& $($mut_)* *existing_child, key, new_key_fragments)
                        },

                        KeyMatch::FirstPrefix => None,

                        KeyMatch::SecondPrefix => {
                            let prefix_length = existing_child.key.len();
                            let new_key_fragments = key_fragments.split(prefix_length);

                            $name(& $($mut_)* *existing_child, key, new_key_fragments)
                        }
                    }
                }
            }
        });
    }
}

get_function!(name: get_mut, mutability: mut);
get_function!(name: get, mutability: );

impl<K, V> TrieNode<K, V> where K: TrieKey {
    /// Create a node with no children.
    fn new(key_fragments: NibbleVec, key: K, value: V) -> TrieNode<K, V> {
        TrieNode {
            key: key_fragments,
            key_value: Some(KeyValue { key: key, value: value }),
            children: no_children![],
            child_count: 0
        }
    }

    /// Add a child at the given index, assuming none exists already.
    fn add_child(&mut self, idx: usize, node: Box<TrieNode<K, V>>) {
        self.children[idx] = Some(node);
        self.child_count += 1;
    }

    /// Remove a child at the given index, if it exists.
    fn take_child(&mut self, idx: usize) -> Option<Box<TrieNode<K, V>>> {
        self.children[idx].take().map(|node| { self.child_count -= 1; node })
    }

    /// Helper function for removing the single child of a node.
    fn take_only_child(&mut self) -> Box<TrieNode<K, V>> {
        for i in 0 .. BRANCH_FACTOR {
            match self.take_child(i) {
                Some(child) => return child,
                None => ()
            }
        }
        unreachable!("node with child_count 1 has no actual children");
    }

    /// Set the key and value of a node.
    fn set_key_value(&mut self, key: K, value: V) {
        self.key_value = Some(KeyValue { key: key, value: value });
    }

    /// Move the value out of a node, whilst checking that its key is as expected.
    /// Can panic (see check_keys).
    fn extract_value(&mut self, key: &K) -> Option<V> {
        self.key_value.take().map(|kv| {
            check_keys(&kv.key, key);
            kv.value
        })
    }

    /// Insert a given key and value below the current node.
    /// Let `key_fragments` be the bits of the key which are valid for insertion *below*
    /// the current node such that the 0th element of `key_fragments` describes the bucket
    /// that this key would be inserted into.
    fn insert(&mut self, key: K, value: V, mut key_fragments: NibbleVec) -> Option<V> {
        // Handle inserts at the root.
        if key_fragments.len() == 0 {
            let result = self.extract_value(&key);
            self.set_key_value(key, value);
            return result;
        }

        let bucket = key_fragments.get(0) as usize;

        match self.children[bucket] {
            // Case 1: No match. Simply insert.
            None => {
                self.add_child(bucket, Box::new(TrieNode::new(key_fragments, key, value)));
                None
            }

            Some(ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    // Case 2: Full key match. Replace existing.
                    KeyMatch::Full => {
                        let result = existing_child.extract_value(&key);
                        existing_child.set_key_value(key, value);
                        result
                    }

                    // Case 3: Partial key match.
                    // Split the existing node's key, insert a new child for the second half of the
                    // key and insert the new key as a new child, with the prefix stripped.
                    KeyMatch::Partial(idx) => {
                        // Split the existing child.
                        existing_child.split(idx);

                        // Insert the new key below the prefix node.
                        let new_key = key_fragments.split(idx);
                        let new_key_bucket = new_key.get(0) as usize;

                        existing_child.add_child(
                            new_key_bucket,
                            Box::new(TrieNode::new(new_key, key, value))
                        );

                        None
                    }

                    // Case 4: Existing key is a prefix.
                    // Strip the prefix and insert below the existing child (recurse).
                    KeyMatch::SecondPrefix => {
                        let prefix_length = existing_child.key.len();
                        let new_key_tail = key_fragments.split(prefix_length);

                        existing_child.insert(key, value, new_key_tail)
                    }

                    // Case 5: Key to insert is a prefix of the existing one.
                    // Split the existing child and place its value below the new one.
                    KeyMatch::FirstPrefix => {
                        existing_child.split(key_fragments.len());
                        existing_child.set_key_value(key, value);

                        None
                    }
                }
            }
        }
    }

    fn remove_recursive(&mut self, key: &K, mut key_fragments: NibbleVec)
        -> (Option<V>, DeleteAction<K, V>) {
        // Handle removals at the root.
        if key_fragments.len() == 0 {
            return (self.extract_value(&key), DoNothing);
        }

        let bucket = key_fragments.get(0) as usize;

        let (value, delete_action) = match self.children[bucket] {
            // Case 1: Not found, nothing to remove.
            None => return (None, DoNothing),

            Some(ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    // Case 2: Node found.
                    KeyMatch::Full => {
                        match existing_child.key_value.take() {
                            // Case 2a: Key found, pass the value up and delete the node.
                            Some(KeyValue { key: ex_key, value }) => {
                                check_keys(key, &ex_key);

                                (Some(value), existing_child.delete_node())
                            }

                            // Case 2b: Key not found, nothing to remove.
                            None => return (None, DoNothing)
                        }
                    }

                    // Case 3: Recurse down.
                    KeyMatch::SecondPrefix => {
                        let prefix_length = existing_child.key.len();
                        let new_key_tail = key_fragments.split(prefix_length);

                        existing_child.remove_recursive(key, new_key_tail)
                    }

                    // Case 4: Not found, nothing to remove.
                    KeyMatch::Partial(_) | KeyMatch::FirstPrefix => return (None, DoNothing)
                }
            }
        };

        // Apply the computed delete action.
        match delete_action {
            Replace(node) => {
                self.children[bucket] = Some(node);
                (value, DoNothing)
            }

            Delete => {
                self.take_child(bucket);

                // The removal of a child could cause this node to be replaced or deleted.
                (value, self.delete_node())
            }

            DoNothing => (value, DoNothing)
        }
    }

    /// Having removed the value from a node, work out if the node itself should be deleted.
    /// Depending on the number of children, this method does one of three things.
    ///     0 children => Delete the node if it is valueless, otherwise DoNothing.
    ///     1 child => Replace the current node by its child.
    ///     2 or more children => DoNothing.
    fn delete_node(&mut self) -> DeleteAction<K, V> {
        match self.child_count {
            0 if self.key_value.is_some() => DoNothing,
            0 => Delete,
            1 => {
                let mut child = self.take_only_child();

                // Join the child's key onto the existing one.
                let new_key = self.key.clone().join(&child.key);

                child.key = new_key;

                Replace(child)
            }
            _ => DoNothing
        }
    }

    /// Split a node at a given index in its key, transforming it into a prefix node of its
    /// previous self.
    fn split(&mut self, idx: usize) {
        // Extract all the parts of the suffix node, starting with the key.
        let key = self.key.split(idx);

        // Key-value.
        let key_value = self.key_value.take();

        // Children.
        let mut children = no_children![];

        for (i, child) in self.children.iter_mut().enumerate() {
            if child.is_some() {
                children[i] = child.take();
            }
        }

        // Child count.
        let child_count = self.child_count;
        self.child_count = 1;

        // Insert the collected items below what is now an empty prefix node.
        let bucket = key.get(0) as usize;
        self.children[bucket] = Some(Box::new(
            TrieNode {
                key: key,
                key_value: key_value,
                children: children,
                child_count: child_count
            }
        ));
    }

    /// Check the integrity of a trie subtree (quite costly).
    /// Return true and the size of the subtree if all checks are successful,
    /// or false and a junk value if any test fails.
    fn check_integrity(&self, prefix: &NibbleVec) -> (bool, usize) {
        let mut sub_tree_size = 0;
        let is_root = prefix.len() == 0;

        // Check that no value-less, non-root nodes have only 1 child.
        if !is_root && self.child_count == 1 && self.key_value.is_none() {
            println!("Value-less node with a single child.");
            return (false, sub_tree_size);
        }

        // Check that all non-root key vector's have length > 1.
        if !is_root && self.key.len() == 0 {
            println!("Key length is 0 at non-root node.");
            return (false, sub_tree_size);
        }

        // Check that the child count matches the actual number of children.
        let child_count = self.children.iter().fold(0, |acc, elem| {
            if elem.is_some() {
                acc + 1
            } else {
                acc
            }
        });

        if child_count != self.child_count {
            println!("Child count error, recorded: {}, actual: {}", self.child_count, child_count);
            return (false, sub_tree_size);
        }

        // Compute the key fragments for this node, according to the trie.
        let trie_key = prefix.clone().join(&self.key);

        // Account for this node in the size check, and check its key.
        match self.key_value {
            Some(ref kv) => {
                sub_tree_size += 1;

                let actual_key = NibbleVec::from_byte_vec(kv.key.encode());

                if trie_key != actual_key {
                    return (false, sub_tree_size);
                }
            }
            None => ()
        }

        // Recursively check children.
        for i in 0 .. BRANCH_FACTOR {
            if let Some(ref child) = self.children[i] {
                match child.check_integrity(&trie_key) {
                    (false, _) => return (false, sub_tree_size),
                    (true, child_size) => sub_tree_size += child_size
                }
            }
        }

        (true, sub_tree_size)
    }
}
