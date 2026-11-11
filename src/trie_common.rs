use crate::TrieNode;
use crate::iter::*;
use crate::traversal::DescendantResult::*;
use crate::{SubTrie, SubTrieMut, Trie, TrieKey};

use nibble_vec::Nibblet;

/// Common functionality available for tries and subtries.
pub trait TrieCommon<'a, K: 'a, V: 'a>: ContainsTrieNode<'a, K, V>
where
    K: TrieKey,
    Self: Sized,
{
    /// Get the key stored at this node, if any.
    #[inline]
    fn key(self) -> Option<&'a K> {
        self.trie_node().key()
    }

    /// Get the value stored at this node, if any.
    #[inline]
    fn value(self) -> Option<&'a V> {
        self.trie_node().value()
    }

    /// Number of key/value pairs stored in this trie.
    fn len(self) -> usize;

    /// Determine if the Trie contains 0 key-value pairs.
    #[inline]
    fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Determine if the trie is a leaf node (has no children).
    #[inline]
    fn is_leaf(self) -> bool {
        self.trie_node().child_count == 0
    }

    /// Return an iterator over the keys and values of the Trie.
    #[inline]
    fn iter(self) -> Iter<'a, K, V> {
        Iter::new(self.trie_node())
    }

    /// Return an iterator over the keys of the Trie.
    #[inline]
    fn keys(self) -> Keys<'a, K, V> {
        Keys::new(self.iter())
    }

    /// Return an iterator over the values of the Trie.
    #[inline]
    fn values(self) -> Values<'a, K, V> {
        Values::new(self.iter())
    }

    /// Return an iterator over the child subtries of this node.
    fn children(self) -> Children<'a, K, V>;

    /// Get the prefix of this node.
    #[inline]
    fn prefix(self) -> &'a Nibblet {
        &self.trie_node().key
    }
}

/// Helper trait for Trie/SubTrie/SubTrieMut, which all contain a trie node.
pub trait ContainsTrieNode<'a, K: 'a, V: 'a>
where
    K: TrieKey,
{
    fn trie_node(self) -> &'a TrieNode<K, V>;
}

/// Regular trie.
impl<'a, K: 'a, V: 'a> ContainsTrieNode<'a, K, V> for &'a Trie<K, V>
where
    K: TrieKey,
{
    #[inline]
    fn trie_node(self) -> &'a TrieNode<K, V> {
        &self.node
    }
}

impl<'a, K: 'a, V: 'a> TrieCommon<'a, K, V> for &'a Trie<K, V>
where
    K: TrieKey,
{
    #[inline]
    fn len(self) -> usize {
        self.length
    }
    #[inline]
    fn children(self) -> Children<'a, K, V> {
        Children::new(self.node.key.clone(), &self.node)
    }
}

/// Subtrie.
impl<'a: 'b, 'b, K: 'a, V: 'a> ContainsTrieNode<'a, K, V> for &'b SubTrie<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn trie_node(self) -> &'a TrieNode<K, V> {
        self.node
    }
}

impl<'a: 'b, 'b, K: 'a, V: 'a> TrieCommon<'a, K, V> for &'b SubTrie<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn len(self) -> usize {
        self.node.compute_size()
    }
    #[inline]
    fn children(self) -> Children<'a, K, V> {
        Children::new(self.prefix.clone(), self.node)
    }
}

/// Mutable subtrie *by value* (consumes the subtrie).
impl<'a, K: 'a, V: 'a> ContainsTrieNode<'a, K, V> for SubTrieMut<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn trie_node(self) -> &'a TrieNode<K, V> {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => node,
            None => self.node,
        }
    }
}

impl<'a, K: 'a, V: 'a> TrieCommon<'a, K, V> for SubTrieMut<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn key(self) -> Option<&'a K> {
        self.node.get(&self.prefix).and_then(TrieNode::key)
    }
    #[inline]
    fn value(self) -> Option<&'a V> {
        self.node.get(&self.prefix).and_then(TrieNode::value)
    }
    /// **Computes** from scratch.
    #[inline]
    fn len(self) -> usize {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => node.compute_size(),
            None => 0,
        }
    }
    #[inline]
    fn is_leaf(self) -> bool {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node)) => node.child_count == 0,
            Some(ExtendKey(_, _, _)) => false,
            None => true,
        }
    }
    #[inline]
    fn iter(self) -> Iter<'a, K, V> {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => Iter::new(node),
            None => Iter::empty(self.node),
        }
    }
    #[inline]
    fn children(self) -> Children<'a, K, V> {
        match self.node.get(&self.prefix) {
            Some(node) => Children::new(self.prefix, node),
            None => Children::empty(self.prefix, self.node),
        }
    }
}

/// Mutable subtrie *by reference* (doesn't consume the subtrie, but limited).
impl<'a: 'b, 'b, K: 'a, V: 'a> ContainsTrieNode<'b, K, V> for &'b SubTrieMut<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn trie_node(self) -> &'b TrieNode<K, V> {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => node,
            None => self.node,
        }
    }
}

impl<'a: 'b, 'b, K: 'a, V: 'a> TrieCommon<'b, K, V> for &'b SubTrieMut<'a, K, V>
where
    K: TrieKey,
{
    #[inline]
    fn key(self) -> Option<&'b K> {
        self.node.get(&self.prefix).and_then(TrieNode::key)
    }
    #[inline]
    fn value(self) -> Option<&'b V> {
        self.node.get(&self.prefix).and_then(TrieNode::value)
    }
    #[inline]
    fn len(self) -> usize {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => node.compute_size(),
            None => 0,
        }
    }
    #[inline]
    fn is_leaf(self) -> bool {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node)) => node.child_count == 0,
            Some(ExtendKey(_, _, _)) => false,
            None => true,
        }
    }
    #[inline]
    fn iter(self) -> Iter<'b, K, V> {
        match self.node.get_raw_descendant(&self.prefix) {
            Some(NoModification(node) | ExtendKey(node, _, _)) => Iter::new(node),
            None => Iter::empty(self.node),
        }
    }
    #[inline]
    fn children(self) -> Children<'b, K, V> {
        match self.node.get(&self.prefix) {
            Some(node) => Children::new(self.prefix.clone(), node),
            None => Children::empty(self.prefix.clone(), self.node),
        }
    }
}
