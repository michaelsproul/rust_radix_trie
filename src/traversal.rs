use {Trie, TrieKey, NibbleVec};
use keys::{match_keys, KeyMatch};

/// Identity macro to allow expansion of the "mutability" token tree.
macro_rules! id {
    ($e:item) => { $e }
}

// FIXME: Use proper doc comments instead of the expanded form (pending macro fix).
macro_rules! make_traversal_trait {
    (
        name: $name:ident,
        trie_ref: $trie_ref:ident,
        mutability: $($mut_:tt)*
    ) => {id!(
#[doc = "Trait capturing a traversal of a trie."]
#[doc = ""]
#[doc = "By providing functions for each of the different cases, it is possible to describe a number"]
#[doc = "of different traversals. For now it's probably best to view the source for `run` to understand"]
#[doc = "how best to implement each function."]
pub trait $name<'a, K, V> where K: TrieKey {
    #[doc = "Key type to be threaded through by `run`, needn't be `K` (is often `&'a K`)."]
    type Key: 'a;
    #[doc = "Value type to be threaded through by `run`, needn't be `V` (is often `()`)."]
    type Value: 'a;
    #[doc = "Type returned by the entire traversal, for insert it's `Option<V>`."]
    type Result;

    fn default_result() -> Self::Result;

    #[allow(unused)]
    fn root_fn(trie: & $($mut_)* Trie<K, V>, key: Self::Key, value: Self::Value) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused)]
    fn no_child_fn
    (
        trie: & $($mut_)* Trie<K, V>, key: Self::Key, value: Self::Value,
        nv: NibbleVec, bucket: usize
    ) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused)]
    fn full_match_fn(trie: & $($mut_)* Trie<K, V>, key: Self::Key, value: Self::Value, nv: NibbleVec)
    -> Self::Result {
        Self::default_result()
    }

    #[allow(unused)]
    fn partial_match_fn
    (
        trie: & $($mut_)* Trie<K, V>, key: Self::Key, value: Self::Value,
        nv: NibbleVec, idx: usize
    ) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused)]
    fn first_prefix_fn(trie: & $($mut_)* Trie<K, V>, key: Self::Key, value: Self::Value, nv: NibbleVec)
    -> Self::Result {
        Self::default_result()
    }

    #[allow(unused)]
    fn action_fn(trie: & $($mut_)* Trie<K, V>, intermediate: Self::Result, bucket: usize)
    -> Self::Result {
        intermediate
    }

    #[doc = "Run the traversal, returning the result."]
    #[doc = ""]
    #[doc = "Let `key_fragments` be the bits of the key which are valid for insertion *below*"]
    #[doc = "the current node such that the 0th element of `key_fragments` describes the bucket"]
    #[doc = "that this key would be inserted into."]
    fn run
    (
        trie: & $($mut_)* Trie<K, V>,
        key: Self::Key,
        value: Self::Value,
        mut key_fragments: NibbleVec
    )
    -> Self::Result {

        if key_fragments.len() == 0 {
            return Self::root_fn(trie, key, value);
        }

        let bucket = key_fragments.get(0) as usize;

        let intermediate = match trie.children[bucket] {
            None => return Self::no_child_fn(trie, key, value, key_fragments, bucket),
            Some(ref $($mut_)* child) => {
                match match_keys(&key_fragments, &child.key) {
                    KeyMatch::Full =>
                        Self::full_match_fn(child, key, value, key_fragments),
                    KeyMatch::Partial(i) =>
                        Self::partial_match_fn(child, key, value, key_fragments, i),
                    KeyMatch::FirstPrefix =>
                        Self::first_prefix_fn(child, key, value, key_fragments),
                    KeyMatch::SecondPrefix => {
                        let new_key = key_fragments.split(child.key.len());
                        Self::run(child, key, value, new_key)
                    }
                }
            }
        };

        Self::action_fn(trie, intermediate, bucket)
    }
});}
}

make_traversal_trait!(name: Traversal, trie_ref: TrieRef, mutability: );
make_traversal_trait!(name: TraversalMut, trie_ref: TrieMut, mutability: mut);
