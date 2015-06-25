use std::fmt::Debug;
use NibbleVec;

/// Trait for types which can be used to key a Radix Trie.
///
/// Types that implement this trait should be convertible to a vector of bytes
/// such that no two instances of the type convert to the same vector. This is essentially
/// serialisation, and may be combined with some serialisation library in the future.
///
/// If a type fails to implement this trait correctly, the Radix Trie will panic upon
/// encountering a conflict. Be careful!
pub trait TrieKey: PartialEq + Eq + Debug {
    /// Encode a value as a vector of bytes.
    fn encode(&self) -> Vec<u8>;
}

/// Key comparison result.
#[derive(Debug)]
pub enum KeyMatch {
    /// The keys match up to the given index.
    Partial(usize),
    /// The first key is a prefix of the second.
    FirstPrefix,
    /// The second key is a prefix of the first.
    SecondPrefix,
    /// The keys match exactly.
    Full
}

/// Compare two Trie keys.
pub fn match_keys(first: &NibbleVec, second: &NibbleVec) -> KeyMatch {
    let min_length = ::std::cmp::min(first.len(), second.len());

    for i in 0..min_length {
        if first.get(i) != second.get(i) {
            return KeyMatch::Partial(i);
        }
    }

    match (first.len(), second.len()) {
        (x, y) if x < y => KeyMatch::FirstPrefix,
        (x, y) if x == y => KeyMatch::Full,
        _ => KeyMatch::SecondPrefix
    }
}

/// Check two keys for equality and panic if they differ.
pub fn check_keys<K>(key1: &K, key2: &K) where K: TrieKey {
    if *key1 != *key2 {
        panic!("multiple-keys with the same bit representation.\n{:?}\n{:?}", key1, key2);
    }
}

/// --- TrieKey Implementations for standard types --- ///

impl<'a> TrieKey for &'a str {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
