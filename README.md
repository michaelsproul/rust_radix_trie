Rust Radix Trie
====

This is a [Radix Trie][radix-wiki] implementation in Rust, building on the lessons learnt from
`TrieMap` and [Sequence Trie][seq-trie].

# Features

* Compressed nodes. Common key prefixes are stored only once.
* Key Generic. Any type that can be serialised as a vector of bytes can be used as a key.
* Safe. No unsafe code (yet).

# To Do

* Implement the Entry API.
* Add Trie-specific methods for prefixes, successors, etc.
* Add iterators.

[radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
[seq-trie]: https://github.com/michaelsproul/rust-sequence-trie
