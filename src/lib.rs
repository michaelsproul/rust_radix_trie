#![feature(box_syntax, box_patterns)]

extern crate nibble_vec;

pub use nibble_vec::NibbleVec;
pub use keys::TrieKey;
use std::fmt::Debug;
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

#[derive(Debug)]
pub struct Trie<K, V> {
    root: TrieNode<K, V>,
    length: usize
}

#[derive(Debug)]
struct TrieNode<K, V> {
    key: NibbleVec,
    key_value: Option<KeyValue<K, V>>,
    children: [Option<Box<TrieNode<K, V>>>; BRANCH_FACTOR],
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

impl<K, V> Trie<K, V> where K: TrieKey, V: Debug {
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

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        let result = self.root.insert(key, value, key_fragments);

        if result.is_none() {
            self.length += 1;
        }

        result
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());

        // Use the TrieNode recursive removal function but ignore its delete action.
        // The root can't be replaced or deleted.
        let (result, _) = self.root.remove_recursive(key, key_fragments);

        result
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        get_mut(&mut self.root, key, key_fragments)
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        get(&self.root, key, key_fragments)
    }
}

/// Identity macro to allow expansion of mutability token tree.
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
            -> Option<&'a $($mut_)* V> where K: TrieKey {

            let bucket = key_fragments.get(0) as usize;

            match trie.children[bucket] {
                None => None,
                Some(box ref $($mut_)* existing_child) => {
                    match match_keys(&key_fragments, &existing_child.key) {
                        KeyMatch::Full => {
                            match existing_child.key_value {
                                Some(ref $($mut_)* kv) => {
                                    assert_eq!(&kv.key, key);
                                    Some(& $($mut_)* kv.value)
                                },
                                None => None
                            }
                        },

                        KeyMatch::Partial(idx) => {
                            let new_key_fragments = key_fragments.split(idx);

                            $name(existing_child, key, new_key_fragments)
                        },

                        KeyMatch::FirstPrefix => None,

                        KeyMatch::SecondPrefix => {
                            let prefix_length = existing_child.key.len();
                            let new_key_fragments = key_fragments.split(prefix_length);

                            $name(existing_child, key, new_key_fragments)
                        }
                    }
                }
            }
        });
    }
}

get_function!(name: get_mut, mutability: mut);
get_function!(name: get, mutability: );

impl<K, V> TrieNode<K, V> where K: TrieKey, V: Debug {
    /// Create a node with no children.
    fn new(key_fragments: NibbleVec, key: K, value: V) -> TrieNode<K, V> {
        TrieNode {
            key: key_fragments,
            key_value: Some(KeyValue { key: key, value: value }),
            children: no_children![],
            child_count: 0
        }
    }

    /// Insert a given key and value below the current node.
    /// Let `key_fragments` be the bits of the key which are valid for insertion *below*
    /// the current node such that the 0th element of `key_fragments` describes the bucket
    /// that this key would be inserted into.
    fn insert(&mut self, key: K, value: V, mut key_fragments: NibbleVec) -> Option<V> {
        let bucket = key_fragments.get(0) as usize;

        match self.children[bucket] {
            // Case 1: No match. Simply insert.
            None => {
                self.children[bucket] = Some(Box::new(TrieNode::new(key_fragments, key, value)));
                self.child_count += 1;
                None
            }

            Some(box ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    // Case 2: Full key match. Replace existing.
                    KeyMatch::Full => {
                        let result = match existing_child.key_value.take() {
                            Some(kv) => {
                                check_keys(&kv.key, &key);
                                Some(kv.value)
                            },
                            None => None
                        };

                        existing_child.key_value = Some(KeyValue { key: key, value: value });

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

                        existing_child.children[new_key_bucket] = Some(Box::new(
                            TrieNode::new(new_key, key, value)
                        ));
                        existing_child.child_count += 1;

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

                        existing_child.key_value = Some(KeyValue { key: key, value: value });

                        None
                    }
                }
            }
        }
    }

    fn remove_recursive(&mut self, key: &K, mut key_fragments: NibbleVec)
        -> (Option<V>, DeleteAction<K, V>) {

        let bucket = key_fragments.get(0) as usize;

        let (value, delete_action) = match self.children[bucket] {
            // Case 1: Not found, nothing to remove.
            None => return (None, DoNothing),

            Some(box ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    // Case 2: Node found.
                    KeyMatch::Full => {
                        match existing_child.key_value.take() {
                            // Case 2a: Key found, pass the value up and delete the node.
                            Some(KeyValue { key: ex_key, value }) => {
                                check_keys(key, &ex_key);

                                println!("At the bottom.");
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

                        println!("Recursing down.");
                        existing_child.remove_recursive(key, new_key_tail)
                    }

                    // Case 4: Not found, nothing to remove.
                    KeyMatch::Partial(_) | KeyMatch::FirstPrefix => return (None, DoNothing)
                }
            }
        };

        println!("value is {:?}, delete action is {:?}", value, delete_action);

        // Apply the computed delete action.
        match delete_action {
            Replace(node) => {
                self.children[bucket] = Some(node);
                (value, DoNothing)
            }

            Delete => {
                self.children[bucket] = None;
                self.child_count -= 1;

                // The removal of a child could cause this node to be replaced or deleted.
                (value, self.delete_node())
            }

            DoNothing => (value, DoNothing)
        }
    }

    /// Having removed the value from a node, work out if the node itself should be deleted.
    /// Depending on the number of children, this method does one of three things.
    ///     0 children => return true
    ///     1 child => compress the child into this node, return false
    ///     2 or more children => return false
    fn delete_node(&mut self) -> DeleteAction<K, V> {

        // Helper function for getting the single child of a node.
        fn get_child<K, V>(node: &mut TrieNode<K, V>) -> Box<TrieNode<K, V>> {
            for i in 0 .. BRANCH_FACTOR {
                match node.children[i].take() {
                    Some(child) => {
                        node.child_count -= 1;
                        return child;
                    }
                    None => ()
                }
            }
            unreachable!("node with child_count 1 has no actual children");
        }

        match self.child_count {
            0 if self.key_value.is_some() => DoNothing,
            0 => Delete,
            1 => {
                let mut child = get_child(self);

                // Join the child's key onto the existing one.
                let mut new_key = self.key.clone();
                new_key.join(&child.key);

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
}
