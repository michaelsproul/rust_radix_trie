use {Trie, TrieNode, NibbleVec};
use keys::*;
use traversal::*;

impl<K, V> Trie<K, V> where K: TrieKey {
    /// Fetch a reference to the given key's corresponding node, if any.
    ///
    /// Note that there is no mutable version of this method, as mutating
    /// subtries directly violates the key-structure of the trie.
    pub fn subtrie<'a>(&'a self, key: &K) -> Option<SubTrie<'a, K, V>> {
        let key_fragments = key.encode();
        self.root.get(&key_fragments).map(|node| {
            SubTrie::new(key_fragments, node)
        })
    }

    pub fn subtrie_mut<'a>(&'a mut self, key: &K) -> Option<SubTrieMut<'a, K, V>> {
        let key_fragments = key.encode();
        let length_ref = &mut self.length;
        self.root.get_mut(&key_fragments).map(move |node| {
            SubTrieMut::new(key_fragments, length_ref, node)
        })
    }

    /// Fetch a reference to the closest ancestor node of the given key.
    ///
    /// If `key` is encoded as byte-vector `b`, return the node `n` in the tree
    /// such that `n`'s key's byte-vector is the longest possible prefix of `b`, and `n`
    /// has a value.
    ///
    /// Invariant: `result.is_some() => result.key_value.is_some()`.
    pub fn get_ancestor<'a>(&'a self, key: &K) -> Option<SubTrie<'a, K, V>> {
        let key_fragments = key.encode();
        self.root.get_ancestor(&key_fragments).map(|node| {
            SubTrie::new(key_fragments, node)
        })
    }

    /// Fetch the closest ancestor *value* for a given key.
    ///
    /// See `get_ancestor` for precise semantics, this is just a shortcut.
    pub fn get_ancestor_value(&self, key: &K) -> Option<&V> {
        self.get_ancestor(key).and_then(|t| t.node.value())
    }
}


#[derive(Debug)]
pub struct SubTrie<'a, K: 'a, V: 'a> where K: TrieKey {
    prefix: NibbleVec,
    node: &'a TrieNode<K, V>,
}

#[derive(Debug)]
pub struct SubTrieMut<'a, K: 'a, V: 'a> where K: TrieKey {
    prefix: NibbleVec,
    length: &'a mut usize,
    node: &'a mut TrieNode<K, V>,
}

pub type SubTrieResult<T> = Result<Option<T>, ()>;

impl <'a, K, V> SubTrie<'a, K, V> where K: TrieKey {
    fn new(prefix: NibbleVec, node: &'a TrieNode<K, V>) -> Self {
        SubTrie {
            prefix: prefix,
            node: node,
        }
    }

    fn get(&self, key: &K) -> SubTrieResult<&V> {
        subtrie_get(&self.prefix, self.node, key)
    }
}

fn subtrie_get<'a, K, V>(prefix: &NibbleVec, node: &'a TrieNode<K, V>, key: &K)
    -> SubTrieResult<&'a V>
    where K: TrieKey
{
    let mut key_enc = key.encode();
    match match_keys(0, prefix, &key_enc) {
        KeyMatch::Full => Ok(node.value()),
        KeyMatch::FirstPrefix => {
            Ok(node.get(&stripped(key_enc, prefix)).and_then(TrieNode::value))
        }
        _ => Err(())
    }
}

impl <'a, K, V> SubTrieMut<'a, K, V> where K: TrieKey {
    fn new(prefix: NibbleVec, length: &'a mut usize, node: &'a mut TrieNode<K, V>) -> Self {
        SubTrieMut {
            prefix: prefix,
            length: length,
            node: node,
        }
    }

    pub fn get(&self, key: &K) -> SubTrieResult<&V> {
        subtrie_get(&self.prefix, &*self.node, key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, ()> {
        let mut key_enc = key.encode();
        match match_keys(0, &self.prefix, &key_enc) {
            KeyMatch::Full => {
                Ok(self.node.replace_value(key, value))
            }
            KeyMatch::FirstPrefix => {
                let previous = self.node.insert(key, value, stripped(key_enc, &self.prefix));
                if previous.is_none() {
                    *self.length += 1;
                }
                Ok(previous)
            }
            _ => Err(())
        }
    }
}

fn stripped(mut key: NibbleVec, prefix: &NibbleVec) -> NibbleVec {
    key.split(prefix.len())
}
