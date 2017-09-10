Rust Radix Trie
====

[![Build Status](https://travis-ci.org/michaelsproul/rust_radix_trie.svg?branch=master)](https://travis-ci.org/michaelsproul/rust_radix_trie)

This is a [Radix Trie][radix-wiki] implementation in Rust, building on the lessons learnt from `TrieMap` and [Sequence Trie][seq-trie]. You can read about my experience implementing this data structure [here][radix-paper].

# Help Wanted, Enquire Within

*Since writing this code I haven't used it in anger (or production) so it is no doubt in need of some maintenance, testing and optimisation love. If you would like to help out, try solving an open issue, optimise something, or just have a poke around! Thanks :)*

# Features

* Compressed nodes. Common key prefixes are stored only once.
* Trie-specific methods to look-up closest ancestors and descendants.
* Key Generic. Any type that can be serialised as a vector of bytes can be used as a key.
* Safe - no unsafe code.

# Documentation

https://docs.rs/radix_trie/

# Usage

Available on [Crates.io][] as [`radix_trie`][radix-crate].

Just add `radix_trie` to the dependencies section of your `Cargo.toml`, like so:

```toml
[dependencies]
radix_trie = "*"
```

# Contributors

Made by:

* Allan Simon ([@allan-simon](https://github.com/allan-simon))
* Andrew Smith ([@andrewcsmith](https://github.com/andrewcsmith))
* Arthur Carcano ([@NougatRillettes](https://github.com/NougatRillettes))
* Michael Sproul ([@michaelsproul](https://github.com/michaelsproul))
* Robin Lambertz ([@roblabla](https://github.com/roblabla))
* Sergey ([@Albibek](https://github.com/Albibek))

# License

MIT License. Copyright © Michael Sproul and contributors 2016.

[radix-wiki]: http://en.wikipedia.org/wiki/Radix_tree
[seq-trie]: https://github.com/michaelsproul/rust_sequence_trie
[radix-paper]: https://michaelsproul.github.io/rust_radix_paper/
[crates.io]: https://crates.io/
[radix-crate]: https://crates.io/crates/radix_trie
