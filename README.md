Rust Patricia Trie
====

This is a Patricia Trie implementation in Rust, building on the lessons learnt from `TrieMap`
and [Sequence Trie][seq-trie].

# Goals

* Key Generic.
* Memory Efficient (compressed nodes).
* Safe - `unsafe` blocks should only be necessary for implementing the Entry API.

[seq-trie]: https://github.com/michaelsproul/rust-sequence-trie
