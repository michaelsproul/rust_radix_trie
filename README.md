Rust Radix Trie
====

[![Build Status](https://travis-ci.org/michaelsproul/rust_radix_trie.svg)](https://travis-ci.org/michaelsproul/rust_radix_trie)

This is a [Radix Trie][radix-wiki] implementation in Rust, building on the lessons learnt from `TrieMap` and [Sequence Trie][seq-trie]. You can read about my experience implementing this data structure [here][radix-paper].

# Help Wanted, Enquire Within

*Since writing this code I haven't used it in anger (or production) so it is no doubt in need of some maintenance, testing and optimisation love. If you would like to help out, try solving an open issue, optimise something (see [TODO](#To Do)), or just have a poke around!*

# Features

* Compressed nodes. Common key prefixes are stored only once.
* Trie-specific methods to look-up closest ancestors and descendants.
* Key Generic. Any type that can be serialised as a vector of bytes can be used as a key.
* Safe - no unsafe code.

# Usage

Available on Crates.io as `radix_trie`.

```toml
[dependencies]
radix_trie = "*"
```

# Documentation

https://docs.rs/radix_trie/

# To Do

* QuickCheck tests.
* Child iterator (easy, done?).
* Optimise:
    + Make the traversals tail recursive so they can be TCO. Remove is the interesting case.
    + Make a `NibbleSlice`? See [paper][radix-paper].
* Implement the Entry API?

# License

MIT License. Copyright © Michael Sproul and contributors 2016.

[radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
[seq-trie]: https://github.com/michaelsproul/rust_sequence_trie
[radix-paper]: https://michaelsproul.github.io/rust_radix_paper/
