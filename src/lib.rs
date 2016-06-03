//! A wonderful, fast, safe, generic radix trie implementation.
//!
//! To get started, see the docs for `Trie` below.

extern crate nibble_vec;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate rand;

pub use nibble_vec::NibbleVec;
pub use keys::TrieKey;
pub use iter::{Iter, Keys, Values};

use keys::check_keys;
use DeleteAction::*;
use traversal::{RefTraversal, TraversalMut, RefTraversalMut};

mod keys;
mod iter;
#[macro_use] pub mod traversal;
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
    /// Key fragments/bits associated with this node, such that joining the keys from all
    /// parent nodes and this node is equal to the bit-encoding of this node's key.
    key: NibbleVec,

    /// The key and value stored at this node.
    key_value: Option<Box<KeyValue<K, V>>>,

    /// The number of values stored in this sub-trie (this node and all descendants).
    length: usize,

    /// The number of children which are Some rather than None.
    child_count: usize,

    /// The children of this node stored such that the first nibble of each child key
    /// dictates the child's bucket.
    children: [Option<Box<Trie<K, V>>>; BRANCH_FACTOR],
}

#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V
}

#[derive(Debug)]
enum DeleteAction<K, V> {
    Replace(Box<Trie<K, V>>),
    Delete,
    DoNothing
}

impl<K, V> DeleteAction<K, V> {
    fn is_delete(&self) -> bool {
        match *self {
            Delete => true,
            _ => false
        }
    }

    fn is_do_nothing(&self) -> bool {
        match *self {
            DoNothing => true,
            _ => false
        }
    }
}

// Public-facing API.
impl<K, V> Trie<K, V> where K: TrieKey {
    /// Create an empty Trie.
    pub fn new() -> Trie<K, V> {
        Trie {
            key: NibbleVec::new(),
            key_value: None,
            children: no_children![],
            child_count: 0,
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

    /// Determine if the trie is a leaf node (has no children).
    pub fn is_leaf(&self) -> bool {
        self.child_count == 0
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

    /// Fetch a reference to the given key's corresponding value, if any.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_node(key).and_then(|t| t.value_checked(key))
    }

    /// Fetch a mutable reference to the given key's corresponding value, if any.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        GetNodeMut::run(self, (), key_fragments).and_then(|t| t.value_checked_mut(key))
    }

    /// Fetch a reference to the given key's corresponding node, if any.
    ///
    /// Note that there is no mutable version of this method, as mutating
    /// subtries directly violates the key-structure of the trie.
    pub fn get_node(&self, key: &K) -> Option<&Trie<K, V>> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        GetNode::run(self, (), key_fragments)
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

    /// Fetch a reference to the closest ancestor node of the given key.
    ///
    /// If `key` is encoded as byte-vector `b`, return the node `n` in the tree
    /// such that `n`'s key's byte-vector is the longest possible prefix of `b`, and `n`
    /// has a value.
    ///
    /// Invariant: `result.is_some() => result.key_value.is_some()`.
    pub fn get_ancestor(&self, key: &K) -> Option<&Trie<K, V>> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        GetAncestor::run(self, (), key_fragments)
    }

    pub fn get_raw_ancestor(&self, key: &K) -> &Trie<K, V> {
        GetRawAncestor::run(self, (), NibbleVec::from_byte_vec(key.encode())).unwrap()
    }

    /// Fetch the closest ancestor *value* for a given key.
    ///
    /// See `get_ancestor` for precise semantics, this is just a shortcut.
    pub fn get_ancestor_value(&self, key: &K) -> Option<&V> {
        self.get_ancestor(key).and_then(|t| t.value())
    }

    /// Fetch the closest descendant for a given key.
    ///
    /// If the key is in the trie, this is the same as `get_node`.
    pub fn get_descendant(&self, key: &K) -> Option<&Trie<K, V>> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        GetDescendant::run(self, (), key_fragments)
    }

    /// Insert the given key-value pair, returning any previous value associated with the key.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        Insert::run(self, (key, value), key_fragments)
    }

    /// Remove and return the value associated with the given key.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());

        // Use the recursive removal function but ignore its delete action.
        // `delete_node` ensures that `Replace(x)` is never returned for the root.
        let (result, action) = Remove::run(self, key, key_fragments);
        debug_assert!(action.is_delete() || action.is_do_nothing());
        result
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
        self.check_integrity_recursive(&NibbleVec::new()).0
    }
}

macro_rules! impl_get_traversal {
    (
        name: $name:ident,
        traversal: $traversal:ident,
        mutability: $($mut_:tt)*
    ) => { id! {

impl<'a, K: 'a, V: 'a> $traversal<'a, K, V> for $name where K: TrieKey {
    type Input = ();
    type Output = Option<&'a $($mut_)* Trie<K, V>>;

    fn default_result() -> Self::Output { None }
    fn match_fn(root: &'a $($mut_)* Trie<K, V>, _: ()) -> Self::Output {
        Some(root)
    }
}

}} // end macro body.
}

enum GetNode {}
enum GetNodeMut {}

impl_get_traversal!(name: GetNode, traversal: RefTraversal, mutability: );
impl_get_traversal!(name: GetNodeMut, traversal: RefTraversalMut, mutability: mut);

/// Traversal type implementing removal.
enum Remove {}

impl<'a, K: 'a, V: 'a> TraversalMut<'a, K, V> for Remove where K: TrieKey {
    type Input = &'a K;
    type Output = (Option<V>, DeleteAction<K, V>);

    fn default_result() -> Self::Output {
        (None, DoNothing)
    }

    fn match_fn(root: &mut Trie<K, V>, key: &K) -> Self::Output {
        (root.take_value(key), root.delete_node())
    }

    fn action_fn
    (
        trie: &mut Trie<K, V>,
        (value, action): (Option<V>, DeleteAction<K, V>),
        bucket: usize
    )
    -> Self::Output {
        // If a value has been removed, reduce the length of this trie.
        if value.is_some() {
            trie.length -= 1;
        }

        // Apply the computed delete action.
        match action {
            Replace(node) => {
                trie.children[bucket] = Some(node);
                (value, DoNothing)
            }
            Delete => {
                trie.take_child(bucket);
                // The removal of a child could cause this node to be replaced or deleted.
                (value, trie.delete_node())
            }
            DoNothing => (value, DoNothing)
        }
    }
}

/// Traversal type implementing insertion.
enum Insert {}

impl<'a, K: 'a, V: 'a> TraversalMut<'a, K, V> for Insert where K: TrieKey {
    type Input = (K, V);
    type Output = Option<V>;

    fn default_result() -> Option<V> {
        None
    }

    // Full key match. Replace existing.
    fn match_fn(trie: &mut Trie<K, V>, (key, value): (K, V)) -> Option<V> {
        trie.replace_value(key, value)
    }

    // No child, insert directly.
    fn no_child_fn(trie: &mut Trie<K, V>, (key, value): (K, V), nv: NibbleVec, bucket: usize)
    -> Option<V> {
        trie.add_child(bucket, Box::new(Trie::with_key_value(nv, key, value)));
        None
    }

    // Partial key match.
    // Split the existing node's key, insert a new child for the second half of the
    // key and insert the new key as a new child, with the prefix stripped.
    fn partial_match_fn
    (
        child: &mut Trie<K, V>, (key, value): (K, V),
        mut key_fragments: NibbleVec,
        idx: usize
    )
    -> Option<V> {
        // Split the existing child.
        child.split(idx);

        // Insert the new key below the prefix node.
        let new_key = key_fragments.split(idx);
        let new_key_bucket = new_key.get(0) as usize;

        child.add_child(
            new_key_bucket,
            Box::new(Trie::with_key_value(new_key, key, value))
        );

        None
    }

    // Key to insert is a prefix of the existing one.
    // Split the existing child and place its value below the new one.
    fn first_prefix_fn(child: &mut Trie<K, V>, (key, value): (K, V), key_fragments: NibbleVec)
    -> Option<V> {
        child.split(key_fragments.len());
        child.add_key_value(key, value);
        None
    }

    fn action_fn(trie: &mut Trie<K, V>, previous_value: Option<V>, _: usize) -> Self::Output {
        // If there's no previous value, increase the length of the trie.
        if previous_value.is_none() {
            trie.length += 1;
        }
        previous_value
    }
}

// Traversal type implementing get_ancestor.
enum GetAncestor {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetAncestor where K: TrieKey {
    type Input = ();
    type Output = Option<&'a Trie<K, V>>;

    fn default_result() -> Self::Output { None }

    fn no_child_fn(trie: &'a Trie<K, V>, _: (), _: NibbleVec, _: usize) -> Self::Output {
        trie.as_value_node()
    }

    fn match_fn(trie: &'a Trie<K, V>, _: ()) -> Self::Output {
        trie.as_value_node()
    }

    fn action_fn(trie: &'a Trie<K, V>, result: Self::Output, _: usize) -> Self::Output {
        result.or_else(|| trie.as_value_node())
    }
}

// Traversal for getting the nearest ancestor, regardless of whether it has a value or not.
enum GetRawAncestor {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetRawAncestor where K: TrieKey {
    type Input = ();
    type Output = Option<&'a Trie<K, V>>;

    fn default_result() -> Self::Output { None }

    fn no_child_fn(trie: &'a Trie<K, V>, _: (), _: NibbleVec, _: usize) -> Self::Output {
        Some(trie)
    }

    fn match_fn(trie: &'a Trie<K, V>, _: ()) -> Self::Output {
        Some(trie)
    }

    fn action_fn(trie: &'a Trie<K, V>, result: Self::Output, _: usize) -> Self::Output {
        result.or(Some(trie))
    }
}

enum GetDescendant {}

impl<'a, K: 'a, V: 'a> RefTraversal<'a, K, V> for GetDescendant where K: TrieKey {
    type Input = ();
    type Output = Option<&'a Trie<K, V>>;

    fn default_result() -> Self::Output { None }
    fn match_fn(trie: &'a Trie<K, V>, _: ()) -> Self::Output { Some(trie) }
    fn first_prefix_fn(trie: &'a Trie<K, V>, _: (), _: NibbleVec) -> Self::Output {
        Some(trie)
    }
}

// Implementation details.
impl<K, V> Trie<K, V> where K: TrieKey {
    /// Create a Trie with no children.
    fn with_key_value(key_fragments: NibbleVec, key: K, value: V) -> Trie<K, V> {
        Trie {
            key: key_fragments,
            key_value: Some(Box::new(KeyValue { key: key, value: value })),
            children: no_children![],
            child_count: 0,
            length: 1
        }
    }

    /// Return true if this node is the root of the entire trie.
    fn is_root(&self) -> bool {
        self.key.len() == 0
    }

    /// Get the value whilst checking a key match.
    fn value_checked(&self, key: &K) -> Option<&V> {
        self.key_value.as_ref().map(|kv| {
            check_keys(&kv.key, key);
            &kv.value
        })
    }

    // Get a mutable value whilst checking a key match.
    fn value_checked_mut(&mut self, key: &K) -> Option<&mut V> {
        self.key_value.as_mut().map(|kv| {
            check_keys(&kv.key, key);
            &mut kv.value
        })
    }

    /// Add a child at the given index, given that none exists there already.
    fn add_child(&mut self, idx: usize, node: Box<Trie<K, V>>) {
        debug_assert!(self.children[idx].is_none());
        self.child_count += 1;
        self.length += node.length;
        self.children[idx] = Some(node);
    }

    /// Remove a child at the given index, if it exists.
    fn take_child(&mut self, idx: usize) -> Option<Box<Trie<K, V>>> {
        self.children[idx].take().map(|node| {
            self.child_count -= 1;
            self.length -= node.length;
            node
        })
    }

    /// Helper function for removing the single child of a node.
    fn take_only_child(&mut self) -> Box<Trie<K, V>> {
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
        self.length += 1;
    }

    /// Move the value out of a node, whilst checking that its key is as expected.
    /// Can panic (see check_keys).
    fn take_value(&mut self, key: &K) -> Option<V> {
        self.key_value.take().map(|kv| {
            check_keys(&kv.key, key);
            self.length -= 1;
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
    fn as_value_node(&self) -> Option<&Trie<K, V>> {
        self.key_value.as_ref().map(|_| self)
    }

    /// Having removed the value from a node, work out if the node itself should be deleted.
    /// Depending on the number of children, this method does one of three things.
    ///     0 children => Delete the node if it is valueless, otherwise DoNothing.
    ///     1 child => Replace the current node by its child if it has no value and isn't the root.
    ///     2 or more children => DoNothing.
    fn delete_node(&mut self) -> DeleteAction<K, V> {
        match self.child_count {
            0 if self.key_value.is_some() => DoNothing,
            0 => Delete,
            1 if self.key_value.is_none() && !self.is_root() => {
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
            Trie {
                key: key,
                key_value: key_value,
                children: children,
                child_count: child_count,
                length: self.length
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
                match child.check_integrity_recursive(&trie_key) {
                    (false, _) => return (false, sub_tree_size),
                    (true, child_size) => sub_tree_size += child_size
                }
            }
        }

        // Check subtree size.
        if self.length != sub_tree_size {
            println!("Subtree size mismatch, recorded: {}, actual: {}", self.length, sub_tree_size);
            return (false, sub_tree_size);
        }

        (true, sub_tree_size)
    }
}
