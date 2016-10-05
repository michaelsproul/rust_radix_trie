//! A wonderful, fast, safe, generic radix trie implementation.
//!
//! To get started, see the docs for `Trie` below.

extern crate nibble_vec;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate rand;

pub use nibble_vec::NibbleVec;
pub use keys::TrieKey;
pub use iter::{Iter, Keys, Values};
pub use subtrie::{SubTrie, SubTrieMut};

use keys::{check_keys, match_keys, KeyMatch};

mod keys;
mod iter;
mod traversal;
mod subtrie;

#[cfg(test)] mod test;
#[cfg(test)] mod qc_test;

const BRANCH_FACTOR: usize = 16;

macro_rules! no_children {
    () => ([
        None, None, None, None,
        None, None, None, None,
        None, None, None, None,
        None, None, None, None
    ])
}

/// Data-structure for storing and querying string-like keys and associated values.
///
/// Any keys which share a common *prefix* are stored below a single copy of that prefix.
/// This saves space, and also allows the longest prefix of any given key to be found.
///
/// You can read more about Radix Tries on [Wikipedia][radix-wiki].
///
/// Lots of the methods on `Trie` return optional values - they can be composed
/// nicely using `Option::and_then`.
///
/// [radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
#[derive(Debug)]
pub struct Trie<K, V> {
    /// The number of values stored in this sub-trie (this node and all descendants).
    length: usize,
    /// The main content of this trie.
    root: TrieNode<K, V>,
}

#[derive(Debug)]
struct TrieNode<K, V> {
    /// Key fragments/bits associated with this node, such that joining the keys from all
    /// parent nodes and this node is equal to the bit-encoding of this node's key.
    key: NibbleVec,

    /// The key and value stored at this node.
    key_value: Option<Box<KeyValue<K, V>>>,

    /// The number of children which are Some rather than None.
    child_count: usize,

    /// The children of this node stored such that the first nibble of each child key
    /// dictates the child's bucket.
    children: [Option<Box<TrieNode<K, V>>>; BRANCH_FACTOR],
}

#[inline(never)]
fn lol_remove<K, V>(trie: &mut TrieNode<K, V>, key: &K) -> Option<V>
    where K: TrieKey
{
    let nv = key.encode();

    if nv.len() == 0 {
        return trie.take_value(key);
    }

    let bucket = nv.get(0) as usize;

    let child = trie.take_child(bucket);

    match child {
        Some(mut child) => {
            let depth = child.key.len();
            if depth == nv.len() {
                let result = child.take_value(key);
                if child.child_count != 0 {
                    // If removing this node's value has made it a value-less node with a
                    // single child, then merge its child.
                    let repl = if child.child_count == 1 {
                        get_merge_child(&mut child)
                    } else {
                        child
                    };
                    trie.add_child(bucket, repl);
                }
                result
            } else {
                rec_remove(trie, child, bucket, key, depth, &nv)
            }
        }
        None => None
    }
}

fn get_merge_child<K, V>(trie: &mut TrieNode<K, V>) -> Box<TrieNode<K, V>> where K: TrieKey {
    let mut child = trie.take_only_child();

    // Join the child's key onto the existing one.
    child.key = trie.key.clone().join(&child.key);

    child
}

/// Remove the key described by `key`.
fn rec_remove<K, V>(parent: &mut TrieNode<K, V>, mut middle: Box<TrieNode<K, V>>, prev_bucket: usize, key: &K, depth: usize, nv: &NibbleVec)
    -> Option<V> where K: TrieKey
{
    let bucket = nv.get(depth) as usize;

    let child = middle.take_child(bucket);
    parent.add_child(prev_bucket, middle);

    match child {
        Some(mut child) => {
            let middle = parent.children[prev_bucket].as_mut().unwrap();
            match match_keys(depth, nv, &child.key) {
                KeyMatch::Full => {
                    let result = child.take_value(key);

                    // If this node has children, keep it.
                    if child.child_count != 0 {
                        // If removing this node's value has made it a value-less node with a
                        // single child, then merge its child.
                        let repl = if child.child_count == 1 {
                            get_merge_child(&mut *child)
                        } else {
                            child
                        };
                        middle.add_child(bucket, repl);
                    }
                    // Otherwise, if the parent node now only has a single child, merge it.
                    else if middle.child_count == 1 && middle.key_value.is_none() {
                        let repl = get_merge_child(middle);
                        *middle = repl;
                    }

                    result
                }
                KeyMatch::SecondPrefix => {
                    let new_depth = depth + child.key.len();
                    rec_remove(middle, child, bucket, key, new_depth, nv)
                }
                _ => None
            }
        }
        None => {
            None
        }
    }
}

#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V
}

// Public-facing API.
impl<K, V> Trie<K, V> where K: TrieKey {
    /// Create an empty Trie.
    pub fn new() -> Trie<K, V> {
        Trie {
            length: 0,
            root: TrieNode {
                key: NibbleVec::new(),
                key_value: None,
                children: no_children![],
                child_count: 0,
            }
        }
    }

    /// Fetch the number of key-value pairs stored in the Trie.
    pub fn len(&self) -> usize {
        self.length
    }

    /// Determine if the Trie contains 0 key-value pairs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Determine if the trie is a leaf node (has no children).
    pub fn is_leaf(&self) -> bool {
        self.root.child_count == 0
    }

    /// Get the key stored at this node, if any.
    pub fn key(&self) -> Option<&K> {
        self.root.key()
    }

    /// Get the value stored at this node, if any.
    pub fn value(&self) -> Option<&V> {
        self.root.value()
    }

    /// Get a mutable reference to the value stored at this node, if any.
    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.root.value_mut()
    }

    /// Fetch a reference to the given key's corresponding value, if any.
    pub fn get(&self, key: &K) -> Option<&V> {
        let key_fragments = key.encode();
        self.root.get(&key_fragments).and_then(|t| t.value_checked(key))
    }

    /// Fetch a mutable reference to the given key's corresponding value, if any.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = key.encode();
        self.root.get_mut(&key_fragments).and_then(|t| t.value_checked_mut(key))
    }

    /// Take a function `f` and apply it to the value stored at `key`.
    ///
    /// If no value is stored at `key`, store `default`.
    pub fn map_with_default<F>(&mut self, key : K, f : F, default: V) where F: Fn(&mut V) {
        {
            if let Some(v) = self.get_mut(&key) {
                f(v);
                return;
            }
        }
        self.insert(key,default);
    }

    // FIXME
    /*
    pub fn get_raw_ancestor(&self, key: &K) -> &TrieNode<K, V> {
        GetRawAncestor::run(self, (), key.encode()).unwrap()
    }
    */

    /// Fetch the closest descendant for a given key.
    ///
    /// If the key is in the trie, this is the same as `get_node`.
    pub fn get_descendant<'a>(&self, key: &K) -> Option<SubTrie<'a, K, V>> {
        // FIXME:
        // let key_fragments = key.encode();
        // GetDescendant::run(self, (), key_fragments)
        None
    }

    /// Insert the given key-value pair, returning any previous value associated with the key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = key.encode();
        let result = self.root.insert(key, value, key_fragments);
        if result.is_none() {
            self.length += 1;
        }
        result
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let removed = lol_remove(&mut self.root, key);
        if removed.is_some() {
            self.length -= 1;
        }
        removed
    }

    /// Return an iterator over the keys and values of the Trie.
    pub fn iter(&self) -> Iter<K, V> {
        Iter::new(self)
    }

    /// Return an iterator over the keys of the Trie.
    pub fn keys(&self) -> Keys<K, V> {
        Keys::new(self.iter())
    }

    /// Return an iterator over the values of the Trie.
    pub fn values(&self) -> Values<K, V> {
        Values::new(self.iter())
    }

    /// Check that the Trie invariants are satisfied - you shouldn't ever have to call this!
    /// Quite slow!
    #[doc(hidden)]
    pub fn check_integrity(&self) -> bool {
        let (ok, length) = self.root.check_integrity_recursive(&NibbleVec::new());
        ok && length == self.length
    }
}

// Traversal type implementing get_ancestor.
/* TODO: fix ancestor traversals 
enum GetAncestor {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetAncestor where K: TrieKey {
    type Input = ();
    type Output = Option<&'a TrieNode<K, V>>;

    fn default_result() -> Self::Output { None }

    fn no_child_fn(trie: &'a TrieNode<K, V>, _: (), _: &NibbleVec, _: usize) -> Self::Output {
        trie.as_value_node()
    }

    fn match_fn(trie: &'a TrieNode<K, V>, _: ()) -> Self::Output {
        trie.as_value_node()
    }

    fn action_fn(trie: &'a TrieNode<K, V>, result: Self::Output, _: usize) -> Self::Output {
        result.or_else(|| trie.as_value_node())
    }
}

// Traversal for getting the nearest ancestor, regardless of whether it has a value or not.
enum GetRawAncestor {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetRawAncestor where K: TrieKey {
    type Input = ();
    type Output = Option<&'a TrieNode<K, V>>;

    fn default_result() -> Self::Output { None }

    fn no_child_fn(trie: &'a TrieNode<K, V>, _: (), _: &NibbleVec, _: usize) -> Self::Output {
        Some(trie)
    }

    fn match_fn(trie: &'a TrieNode<K, V>, _: ()) -> Self::Output {
        Some(trie)
    }

    fn action_fn(trie: &'a TrieNode<K, V>, result: Self::Output, _: usize) -> Self::Output {
        result.or(Some(trie))
    }
}

enum GetDescendant {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetDescendant where K: TrieKey {
    type Input = ();
    type Output = Option<&'a TrieNode<K, V>>;

    fn default_result() -> Self::Output { None }
    fn match_fn(trie: &'a TrieNode<K, V>, _: ()) -> Self::Output { Some(trie) }
    fn first_prefix_fn(trie: &'a TrieNode<K, V>, _: (), _: NibbleVec) -> Self::Output {
        Some(trie)
    }
}
*/

// Implementation details.
impl<K, V> TrieNode<K, V> where K: TrieKey {
    /// Create a TrieNode with no children.
    fn with_key_value(key_fragments: NibbleVec, key: K, value: V) -> TrieNode<K, V> {
        TrieNode {
            key: key_fragments,
            key_value: Some(Box::new(KeyValue { key: key, value: value })),
            children: no_children![],
            child_count: 0,
        }
    }

    /// Get the key stored at this node, if any.
    pub fn key(&self) -> Option<&K> {
        self.key_value.as_ref().map(|kv| &kv.key)
    }

    /// Get the value stored at this node, if any.
    pub fn value(&self) -> Option<&V> {
        self.key_value.as_ref().map(|kv| &kv.value)
    }

    /// Get a mutable reference to the value stored at this node, if any.
    pub fn value_mut(&mut self) -> Option<&mut V> {
        self.key_value.as_mut().map(|kv| &mut kv.value)
    }

    /// Get the value whilst checking a key match.
    fn value_checked(&self, key: &K) -> Option<&V> {
        self.key_value.as_ref().map(|kv| {
            check_keys(&kv.key, key);
            &kv.value
        })
    }

    /// Get a mutable value whilst checking a key match.
    fn value_checked_mut(&mut self, key: &K) -> Option<&mut V> {
        self.key_value.as_mut().map(|kv| {
            check_keys(&kv.key, key);
            &mut kv.value
        })
    }

    /// Add a child at the given index, given that none exists there already.
    fn add_child(&mut self, idx: usize, node: Box<TrieNode<K, V>>) {
        debug_assert!(self.children[idx].is_none());
        self.child_count += 1;
        self.children[idx] = Some(node);
    }

    /// Remove a child at the given index, if it exists.
    fn take_child(&mut self, idx: usize) -> Option<Box<TrieNode<K, V>>> {
        self.children[idx].take().map(|node| {
            self.child_count -= 1;
            node
        })
    }

    /// Helper function for removing the single child of a node.
    fn take_only_child(&mut self) -> Box<TrieNode<K, V>> {
        debug_assert!(self.child_count == 1);
        for i in 0 .. BRANCH_FACTOR {
            if let Some(child) = self.take_child(i) {
                return child;
            }
        }
        unreachable!("node with child_count 1 has no actual children");
    }

    /// Set the key and value of a node, given that it currently lacks one.
    fn add_key_value(&mut self, key: K, value: V) {
        debug_assert!(self.key_value.is_none());
        self.key_value = Some(Box::new(KeyValue { key: key, value: value }));
    }

    /// Move the value out of a node, whilst checking that its key is as expected.
    /// Can panic (see check_keys).
    fn take_value(&mut self, key: &K) -> Option<V> {
        self.key_value.take().map(|kv| {
            check_keys(&kv.key, key);
            kv.value
        })
    }

    /// Replace a value, returning the previous value if there was one.
    fn replace_value(&mut self, key: K, value: V) -> Option<V> {
        let previous = self.take_value(&key);
        self.add_key_value(key, value);
        previous
    }

    /// Get a reference to this node if it has a value.
    fn as_value_node(&self) -> Option<&TrieNode<K, V>> {
        self.key_value.as_ref().map(|_| self)
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
                child_count: child_count,
            }
        ));
    }

    /// Check the integrity of a trie subtree (quite costly).
    /// Return true and the size of the subtree if all checks are successful,
    /// or false and a junk value if any test fails.
    fn check_integrity_recursive(&self, prefix: &NibbleVec) -> (bool, usize) {
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
        let child_count = self.children.iter().fold(0, |acc, e| acc + (e.is_some() as usize));

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

                let actual_key = kv.key.encode();

                if trie_key != actual_key {
                    return (false, sub_tree_size);
                }
            }
            None => ()
        }

        // Recursively check children.
        for i in 0 .. BRANCH_FACTOR {
            if let Some(ref child) = self.children[i] {
                match child.check_integrity_recursive(&trie_key) {
                    (false, _) => return (false, sub_tree_size),
                    (true, child_size) => sub_tree_size += child_size
                }
            }
        }

        (true, sub_tree_size)
    }
}
