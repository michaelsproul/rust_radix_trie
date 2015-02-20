#![feature(box_syntax, box_patterns)]

extern crate nibble_vec;

pub use nibble_vec::NibbleVec;

use std::fmt::Debug;

#[cfg(test)]
mod test;

const BRANCH_FACTOR: usize = 16;

macro_rules! no_children {
    () => ([
        None, None, None, None,
        None, None, None, None,
        None, None, None, None,
        None, None, None, None
    ])
}

#[derive(Debug)]
pub struct Trie<K, V> {
    root: TrieNode<K, V>,
    length: usize
}

#[derive(Debug)]
struct TrieNode<K, V> {
    key: NibbleVec,
    key_value: Option<KeyValue<K, V>>,
    children: [Option<Box<TrieNode<K, V>>>; BRANCH_FACTOR],
    child_count: usize
}

#[derive(Debug)]
struct KeyValue<K, V> {
    key: K,
    value: V
}

pub trait TrieKey: PartialEq + Eq + Debug {
    fn encode(&self) -> Vec<u8>;
}

impl<'a> TrieKey for &'a str {
    fn encode(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl<K, V> Trie<K, V> where K: TrieKey {
    pub fn new() -> Trie<K, V> {
        Trie {
            root: TrieNode {
                key: NibbleVec::new(),
                key_value: None,
                children: no_children![],
                child_count: 0
            },
            length: 0
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        self.root.insert(key, value, key_fragments)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        self.root.get_mut(key, key_fragments)
    }
}

impl<K, V> TrieNode<K, V> where K: TrieKey {
    /// Create a node with no children.
    fn new(key_fragments: NibbleVec, key: K, value: V) -> TrieNode<K, V> {
        TrieNode {
            key: key_fragments,
            key_value: Some(KeyValue { key: key, value: value }),
            children: no_children![],
            child_count: 0
        }
    }

    /// Get the value for the given key and key fragments.
    fn get_mut(&mut self, key: &K, mut key_fragments: NibbleVec) -> Option<&mut V> {
        let bucket = key_fragments.get(0) as usize;

        match self.children[bucket] {
            None => None,
            Some(box ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    KeyMatch::Full => {
                        match existing_child.key_value {
                            Some(ref mut kv) => {
                                assert_eq!(&kv.key, key);
                                Some(&mut kv.value)
                            },
                            None => None
                        }
                    },

                    KeyMatch::Partial(idx) => {
                        let new_key_fragments = key_fragments.split(idx);

                        existing_child.get_mut(key, new_key_fragments)
                    },

                    KeyMatch::FirstPrefix => None,

                    KeyMatch::SecondPrefix => {
                        let prefix_length = existing_child.key.len();
                        let new_key_fragments = key_fragments.split(prefix_length);

                        existing_child.get_mut(key, new_key_fragments)
                    }
                }
            }
        }
    }

    /// Insert a given key and value below the current node.
    /// Let `key_fragments` be the bits of the key which are valid for insertion *below*
    /// the current node such that the 0th element of `key_fragments` describes the bucket
    /// that this key would be inserted into.
    fn insert(&mut self, key: K, value: V, mut key_fragments: NibbleVec) -> Option<V> {
        let bucket = key_fragments.get(0) as usize;

        match self.children[bucket] {
            // Case 1: No match. Simply insert.
            None => {
                self.children[bucket] = Some(Box::new(TrieNode::new(key_fragments, key, value)));
                None
            }

            Some(box ref mut existing_child) => {
                match match_keys(&key_fragments, &existing_child.key) {
                    // Case 2: Full key match. Replace existing.
                    KeyMatch::Full => {
                        existing_child.key_value = Some(KeyValue { key: key, value: value });
                        None
                    }

                    // Case 3: Partial key match.
                    // Split the existing node's key, insert a new child for the second half of the
                    // key and insert the new key as a new child, with the prefix stripped.
                    KeyMatch::Partial(idx) => {
                        // Split the existing child.
                        existing_child.split(idx);

                        // Insert the new key below the prefix node.
                        let new_key = key_fragments.split(idx);
                        let new_key_bucket = new_key.get(0) as usize;

                        existing_child.children[new_key_bucket] = Some(Box::new(
                            TrieNode::new(new_key, key, value)
                        ));

                        None
                    }

                    // Case 4: Existing key is a prefix.
                    // Strip the prefix and insert below the existing child (recurse).
                    KeyMatch::SecondPrefix => {
                        let prefix_length = existing_child.key.len();
                        let new_key_tail = key_fragments.split(prefix_length);

                        existing_child.insert(key, value, new_key_tail)
                    }

                    // Case 5: Key to insert is a prefix of the existing one.
                    // Split the existing child and place its value below the new one.
                    KeyMatch::FirstPrefix => {
                        existing_child.split(key_fragments.len());

                        existing_child.key_value = Some(KeyValue { key: key, value: value });

                        None
                    }
                }
            }
        }
    }

    /// Split a node at a given index in its key, transforming it into a prefix node of its
    /// previous self.
    fn split(&mut self, idx: usize) {
        // Extract all the parts of the suffix node, starting with the key.
        let key = self.key.split(idx);

        // Key-value.
        let key_value = self.key_value.take();

        // Children.
        let mut children = no_children![];

        for (i, child) in self.children.iter_mut().enumerate() {
            if child.is_some() {
                children[i] = child.take();
            }
        }

        // Child count.
        let child_count = self.child_count;
        self.child_count = 1;

        // Insert the collected items below what is now an empty prefix node.
        let bucket = key.get(0) as usize;
        self.children[bucket] = Some(Box::new(
            TrieNode {
                key: key,
                key_value: key_value,
                children: children,
                child_count: child_count
            }
        ));
    }
}

enum KeyMatch {
    /// The keys match up to the given index.
    Partial(usize),
    /// The first key is a prefix of the second.
    FirstPrefix,
    /// The second key is a prefix of the first.
    SecondPrefix,
    /// The keys match exactly.
    Full
}

fn match_keys(first: &NibbleVec, second: &NibbleVec) -> KeyMatch {
    let min_length = std::cmp::min(first.len(), second.len());

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

