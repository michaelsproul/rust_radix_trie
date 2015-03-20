Rust Radix Trie
====

[![Build Status](https://travis-ci.org/michaelsproul/rust_radix_trie.svg)](https://travis-ci.org/michaelsproul/rust_radix_trie)

This is a [Radix Trie][radix-wiki] implementation in Rust, building on the lessons learnt from
`TrieMap` and [Sequence Trie][seq-trie].

# Features

* Compressed nodes. Common key prefixes are stored only once.
* Key Generic. Any type that can be serialised as a vector of bytes can be used as a key.
* Safe. No unsafe code (yet).

# Usage

Available on Crates.io as `radix_trie`.

```toml
[dependencies]
radix_trie = "*"
```

# Documentation

https://michaelsproul.github.io/rust_radix_trie/

# To Do

* Add Trie-specific methods for prefixes, successors, etc.
* Implement the Entry API.
* Add iterators.

[radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
[seq-trie]: https://github.com/michaelsproul/rust-sequence-trie

# License

MIT License. Copyright (c) Michael Sproul 2015.
