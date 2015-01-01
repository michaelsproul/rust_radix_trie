extern crate nibble_vec;

use nibble_vec::NibbleVec;

static const BRANCH_FACTOR: uint = 16;

pub struct Trie<K, V> {
    root: TrieNode<K, V>,
    length: uint
}

struct TrieNode<K, V> {
    key: NibbleVec,
    key_length: uint
    key_value: Option<KeyValue<K, V>>
    children: [Option<Box<TrieNode<V>>>; BRANCH_FACTOR]
    child_count: uint
}

struct KeyValue<K, V> {
    key: K,
    value: V
}

pub trait TrieKey {
    fn encode(&self) -> Vec<u8>;
}

impl<K: TrieKey, V> Trie<K, V> {
    pub fn len(&self) -> uint {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_fragments = NibbleVec::from_byte_vec(key.encode());
        self.root.insert(key_fragments, value, 0);
    }

    pub fn get(&self, key: &V) -> Option<&V> {

    }
}

impl<V> TrieNode<V> {
    fn insert(&mut self, key: K, value: V, key_fragments: NibbleVec, depth: uint) -> Option<V> {
        // Compute the index of the child in the current layer where this value could be inserted.
        let index = key_fragments.get(depth);

        // If there's no child at this depth, insert the value.
        if self.children[index].is_none() {
            self.children[index] = Some(
                TrieNode {
                    key: key_fragments,
                    key_length: key_fragments.len(),
                    key_value: KeyValue {
                        key: key,
                        value: value
                    },
                    children: [None; BRANCH_FACTOR],
                    child_count: 0
                }
            )
            return None;
        }
    }
}

