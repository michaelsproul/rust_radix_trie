//! Customisable, user-implementable traversals for tries.
//!
//! This module contains 4 traits that allow you to write your own recursive trie traversals. Each
//! trait handles the key-splitting logic and result handling required to write a traversal. All of
//! the library's important operations are implemented as traversals (insert, remove, get_ancestor,
//! etc).
//!
//! The trait is basically a massively higher-order function. The `Input` type allows you
//! to define values to be passed down the traversal as it proceeds, while the `Output` type
//! allows you to propogate a value back up the call stack. Look at the source for `run`.
//!
//! # Which Trait?
//!
//! * `Traversal` - immutable trie, references not allowed in `Output` type.
//! * `RefTraversal` - immutable trie, references allowed in `Output` type.
//! * `TraversalMut` - mutable trie, references not allowed in `Output` type.
//! * `RefTraversalMut` - mutable trie, references allowed in `Output` type, no `action_fn`.

use {Trie, TrieKey, NibbleVec};
use keys::{match_keys, KeyMatch};

/// Identity macro to allow expansion of the "mutability" token tree.
#[macro_export]
macro_rules! id {
    ($e:item) => { $e }
}

macro_rules! if_else {
    (false, $x:expr, $y:expr) => { $y };
    (true, $x:expr, $y:expr) => { $x };
}

// NOTE: Use () default input type once `associated_type_defaults` stabilises.
macro_rules! make_traversal_trait {
    (
        name: $name:ident,
        trie_type: $trie_type:ty,
        with_action: $with_action:tt,
        mutability: $($mut_:tt)*
    ) => { id! {
pub trait $name<'a, K: 'a, V: 'a> where K: TrieKey {
    type Input: 'a;
    type Output;

    fn default_result() -> Self::Output;

    #[allow(unused)]
    fn match_fn(trie: $trie_type, input: Self::Input) -> Self::Output {
        Self::default_result()
    }

    #[allow(unused)]
    fn no_child_fn
    (
        trie: $trie_type, input: Self::Input,
        nv: NibbleVec, bucket: usize
    ) -> Self::Output {
        Self::default_result()
    }

    #[doc = "Defaults to `match_fn`."]
    #[allow(unused)]
    fn child_match_fn(child: $trie_type, input: Self::Input, nv: NibbleVec) -> Self::Output {
        Self::match_fn(child, input)
    }

    #[allow(unused)]
    fn partial_match_fn(child: $trie_type, input: Self::Input, nv: NibbleVec, idx: usize)
    -> Self::Output {
        Self::default_result()
    }

    #[allow(unused)]
    fn first_prefix_fn(trie: $trie_type, input: Self::Input, nv: NibbleVec) -> Self::Output {
        Self::default_result()
    }

    // NOTE: Don't generate action_fn at all (cf. Rust issue #4621).
    #[doc = "Note: this function isn't called in a `RefTraversalMut`."]
    #[allow(unused)]
    fn action_fn(trie: $trie_type, intermediate: Self::Output, bucket: usize) -> Self::Output {
        intermediate
    }

    #[doc = "Run the traversal, returning the result."]
    fn run(trie: $trie_type, input: Self::Input, mut key_fragments: NibbleVec) -> Self::Output {
        if key_fragments.len() == 0 {
            return Self::match_fn(trie, input);
        }

        let bucket = key_fragments.get(0) as usize;

        let intermediate = match trie.children[bucket] {
            None => return Self::no_child_fn(trie, input, key_fragments, bucket),
            Some(ref $($mut_)* child) => {
                match match_keys(&key_fragments, &child.key) {
                    KeyMatch::Full =>
                        Self::child_match_fn(child, input, key_fragments),
                    KeyMatch::Partial(i) =>
                        Self::partial_match_fn(child, input, key_fragments, i),
                    KeyMatch::FirstPrefix =>
                        Self::first_prefix_fn(child, input, key_fragments),
                    KeyMatch::SecondPrefix => {
                        let new_key = key_fragments.split(child.key.len());
                        Self::run(child, input, new_key)
                    }
                }
            }
        };

        if_else!($with_action, Self::action_fn(trie, intermediate, bucket), intermediate)
    }
} // end trait.
}} // end macro body.
}

make_traversal_trait!(name: Traversal, trie_type: &Trie<K, V>, with_action: true, mutability: );
make_traversal_trait!(name: RefTraversal, trie_type: &'a Trie<K, V>, with_action: true, mutability: );

make_traversal_trait!(name: TraversalMut, trie_type: &mut Trie<K, V>, with_action: true, mutability: mut);
make_traversal_trait!(name: RefTraversalMut, trie_type: &'a mut Trie<K, V>, with_action: false, mutability: mut);
