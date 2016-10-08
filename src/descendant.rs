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
