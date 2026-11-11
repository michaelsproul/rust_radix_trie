//! Iterators over values whose encoded keys prefix a query.

#[cfg(doc)]
use crate::Trie;
use crate::TrieNode;
use nibble_vec::Nibblet;
use std::iter::FusedIterator;

/// Values whose encoded keys prefix a query, in shortest-prefix-first order.
///
/// Used by [`Trie::get_all_ancestor_values`].
///
/// Owns the encoded query, so that the original query does not need to outlive the iterator.
pub struct AllAncestorValues<'a, K, V> {
    key: Nibblet,
    inner: AncestorValues<'a, K, V>,
}

impl<'a, K, V> AllAncestorValues<'a, K, V> {
    pub(crate) fn new(root: &'a TrieNode<K, V>, key: Nibblet) -> Self {
        Self {
            key,
            inner: AncestorValues::new(root),
        }
    }
}

impl<'a, K, V> Iterator for AllAncestorValues<'a, K, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(
            self.key.len(),
            |i| self.key.get(i),
            |depth, node| (0..node.key.len()).all(|i| self.key.get(depth + i) == node.key.get(i)),
        )
    }
}

impl<K, V> FusedIterator for AllAncestorValues<'_, K, V> {}

/// Specialization of [`AllAncestorValues`] when the key is plain bytes to avoid owning them and
/// instead work with a slice.
pub struct AllAncestorValuesBytes<'a, 'q, V> {
    key: &'q [u8],
    inner: AncestorValues<'a, Vec<u8>, V>,
}

impl<'a, 'q, V> AllAncestorValuesBytes<'a, 'q, V> {
    pub(crate) fn new(root: &'a TrieNode<Vec<u8>, V>, key: &'q [u8]) -> Self {
        Self {
            key,
            inner: AncestorValues::new(root),
        }
    }

    /// Read the query byte slice `key` at zero-based nibble index `i`.
    /// Even indices select the high nibble; odd indices select the low nibble.
    /// The result is in `0..16`, matching the trie's encoded nibble order.
    #[inline]
    fn byte_nibble(key: &[u8], i: usize) -> u8 {
        let byte = key[i / 2];
        if i % 2 == 0 { byte >> 4 } else { byte & 0x0f }
    }

    /// Check whether `node`'s compressed edge matches the query byte slice `key`
    /// starting at `depth`, measured in nibbles from the start of the query.
    /// Used as the edge-matching callback for [`AncestorValues::next`].
    ///
    /// The caller must ensure that the entire edge fits in the remaining query
    /// and that the query prefix before `depth` has already matched the path to
    /// `node`'s parent. Matching uses byte comparisons for aligned edges and
    /// odd-depth leaf suffixes, falling back to nibble comparisons for odd-depth
    /// internal edges. Padding after an odd-length edge is ignored.
    #[inline]
    fn byte_prefix_matches(key: &[u8], depth: usize, node: &TrieNode<Vec<u8>, V>) -> bool {
        let label = &node.key;
        let offset = depth / 2;
        if depth % 2 == 0 {
            // Aligned labels can use a bulk byte comparison. The low nibble of an
            // odd label's final byte is padding, not part of the edge.
            let bytes = label.as_bytes();
            let whole = label.len() / 2;
            let remaining = &key[offset..];
            return remaining[..whole] == bytes[..whole]
                && (label.len() % 2 == 0 || remaining[whole] >> 4 == bytes[whole] >> 4);
        }
        if node.child_count == 0 {
            // A leaf's original key is byte-aligned even when its edge is not.
            // Compare only the remaining suffix, plus the already-matched high
            // nibble at `offset`. Never rescan whole prefixes at internal nodes.
            return node
                .key_value
                .as_ref()
                .is_some_and(|kv| key[offset..].starts_with(&kv.key[offset..]));
        }
        // Odd-depth internal edges are shifted relative to the query's bytes.
        (0..label.len()).all(|i| Self::byte_nibble(key, depth + i) == label.get(i))
    }
}

impl<'a, V> Iterator for AllAncestorValuesBytes<'a, '_, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next(
            self.key.len() * 2,
            |i| Self::byte_nibble(self.key, i),
            |depth, node| Self::byte_prefix_matches(self.key, depth, node),
        )
    }
}

impl<V> FusedIterator for AllAncestorValuesBytes<'_, '_, V> {}

/// Shared traversal state for yielding stored ancestor values, shortest prefix first.
///
/// Used by [`AllAncestorValues`] and [`AllAncestorValuesBytes`] to follow only
/// the query's path through compressed edges, skipping nodes without values.
/// The query is not stored here: each call to `next` supplies its nibble length,
/// nibble access, and edge-matching logic for the wrapper's query representation.
struct AncestorValues<'a, K, V> {
    /// Last matched node, from which traversal resumes; initially the root.
    /// `None` means traversal has terminated and no further values can be yielded.
    node: Option<&'a TrieNode<K, V>>,
    /// Number of query nibbles matched through the current node, including its
    /// compressed edge.
    depth: usize,
    /// Whether the root has been checked for a value, so an empty-key value is
    /// yielded at most once before descending into children.
    root_visited: bool,
}

impl<'a, K, V> AncestorValues<'a, K, V> {
    fn new(root: &'a TrieNode<K, V>) -> Self {
        Self {
            node: Some(root),
            depth: 0,
            root_visited: false,
        }
    }

    /// Iterator implementation that advances to the next stored value along the query's path,
    /// shortest prefix first.
    /// Returns `None` permanently once the path ends or an edge fails to match.
    ///
    /// - `key_len`: total length of the encoded query in nibbles, not bytes.
    /// - `nibble`: function that returns the query nibble at a zero-based index, with a value
    ///   in `0..16`, to select the next child. Called only below `key_len`.
    /// - `matches`: function that checks whether a child's entire compressed edge matches the
    ///   query starting at the supplied nibble depth. Receives that depth and the child node,
    ///   and is called only after the edge is known to fit within the remaining query.
    #[inline]
    fn next(
        &mut self,
        key_len: usize,
        nibble: impl Fn(usize) -> u8,
        matches: impl Fn(usize, &TrieNode<K, V>) -> bool,
    ) -> Option<&'a V> {
        if !self.root_visited {
            self.root_visited = true;
            if let Some(kv) = self.node?.key_value.as_ref() {
                return Some(&kv.value);
            }
        }

        while let Some(node) = self.node.take() {
            if self.depth == key_len {
                return None;
            }
            let child = node.children[nibble(self.depth) as usize].as_deref()?;
            if child.key.len() > key_len - self.depth {
                return None;
            }
            if !matches(self.depth, child) {
                return None;
            }
            self.depth += child.key.len();
            self.node = Some(child);
            if let Some(kv) = child.key_value.as_ref() {
                return Some(&kv.value);
            }
        }
        None
    }
}
