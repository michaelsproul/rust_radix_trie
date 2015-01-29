#![feature(box_syntax)]

extern crate nibble_vec;

pub use nibble_vec::NibbleVec;

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

pub trait TrieKey {
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

    pub fn get(&self, key: &K) -> Option<&V> {
        None
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
                        let existing_key_tail = existing_child.key.split(idx);

                        // Take the existing key's value and children.
                        let existing_key_value = existing_child.key_value.take();

                        let mut existing_key_children = no_children![];

                        for (i, child) in existing_child.children.iter_mut().enumerate() {
                            if child.is_some() {
                                existing_key_children[i] = child.take();
                            }
                        }

                        let existing_key_child_count = existing_child.child_count;
                        existing_child.child_count = 0;

                        // Insert the existing key below the prefix node.
                        let existing_key_bucket = existing_key_tail.get(0) as usize;
                        existing_child.children[existing_key_bucket] = Some(Box::new(
                            TrieNode {
                                key: existing_key_tail,
                                key_value: existing_key_value,
                                children: existing_key_children,
                                child_count: existing_key_child_count
                            }
                        ));

                        // Insert the new key below the prefix node.
                        let new_key = key_fragments.split(idx);
                        let new_key_bucket = new_key.get(0) as usize;
                        existing_child.children[new_key_bucket] = Some(Box::new(
                            TrieNode::new(new_key, key, value)
                        ));

                        None
                    }

                    // Case 4: Prefix match.
                    // Strip the prefix and insert below the existing child (recurse).
                    KeyMatch::Prefix => {
                        let prefix_length = existing_child.key.len();
                        let new_key_tail = key_fragments.split(prefix_length);

                        existing_child.insert(key, value, new_key_tail)
                    }
                }
            }
        }
    }
}

enum KeyMatch {
    /// Partial match, containing the first index that differs.
    Partial(usize),
    Prefix,
    Full
}

fn match_keys(first: &NibbleVec, second: &NibbleVec) -> KeyMatch {
    println!("matching on keys: {:?} and {:?}", first, second);
    let min_length = std::cmp::min(first.len(), second.len());

    for i in range(0, min_length) {
        if first.get(i) != second.get(i) {
            return KeyMatch::Partial(i);
        }
    }

    if first.len() > second.len() {
        KeyMatch::Prefix
    } else if first.len() == second.len(){
        KeyMatch::Full
    } else {
        unreachable!("trie invariant broken")
    }
}

