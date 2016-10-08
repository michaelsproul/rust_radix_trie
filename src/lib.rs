//! A wonderful, fast, safe, generic radix trie implementation.
//!
//! To get started, see the docs for `Trie` below.

#![warn(missing_docs)]

extern crate nibble_vec;
#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate rand;

pub use nibble_vec::NibbleVec;
pub use keys::TrieKey;

#[macro_use] mod macros;
mod keys;
mod iter;
mod traversal;
mod trie;
mod subtrie;
mod trie_node;

#[cfg(test)] mod test;
#[cfg(test)] mod qc_test;

const BRANCH_FACTOR: usize = 16;

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
    node: TrieNode<K, V>,
}

#[derive(Debug)]
struct TrieNode<K, V> {
    /// Key fragments/bits associated with this node, such that joining the keys from all
    /// parent nodes and this node is equal to the bit-encoding of this node's key.
    key: NibbleVec,

    /// The key and value stored at this node.
    // TODO: consider storing the key-value unboxed.
    key_value: Option<Box<KeyValue<K, V>>>,

    /// The number of children which are Some rather than None.
    child_count: usize,

    /// The children of this node stored such that the first nibble of each child key
    /// dictates the child's bucket.
    children: [Option<Box<TrieNode<K, V>>>; BRANCH_FACTOR],
}

#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V
}

/// Immutable view of a sub-tree a larger trie.
#[derive(Debug)]
pub struct SubTrie<'a, K: 'a, V: 'a> {
    prefix: NibbleVec,
    node: &'a TrieNode<K, V>,
}

/// Mutable view of a sub-tree of a larger trie.
#[derive(Debug)]
pub struct SubTrieMut<'a, K: 'a, V: 'a> {
    prefix: NibbleVec,
    length: &'a mut usize,
    node: &'a mut TrieNode<K, V>,
}

/// Wrapper for subtrie lookup results.
///
/// When fetching from a subtrie, if the prefix is wrong you'll get an `Err(())`.
/// Otherwise you'll get an `Ok(_)`, where the contained option value is what would ordinarily
/// be returned by get/insert/whatever.
pub type SubTrieResult<T> = Result<Option<T>, ()>;
