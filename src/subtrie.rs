use {TrieNode, SubTrie, SubTrieMut, SubTrieResult, NibbleVec};
use keys::*;

impl <'a, K, V> SubTrie<'a, K, V> where K: TrieKey {
    /// Look up the value for the given key, which should be an extension of this subtrie's key.
    pub fn get(&self, key: &K) -> SubTrieResult<&V> {
        subtrie_get(&self.prefix, self.node, key)
    }

    /// Compute the size of this subtrie.
    ///
    /// This isn't a constant time operation and involves a full traversal of the subtrie.
    pub fn len(&self) -> usize {
        subtrie_size(&self.node)
    }

    generate_trie_node_methods!();
}

fn subtrie_get<'a, K, V>(prefix: &NibbleVec, node: &'a TrieNode<K, V>, key: &K)
    -> SubTrieResult<&'a V>
    where K: TrieKey
{
    let key_enc = key.encode();
    match match_keys(0, prefix, &key_enc) {
        KeyMatch::Full => Ok(node.value()),
        KeyMatch::FirstPrefix => {
            Ok(node.get(&stripped(key_enc, prefix)).and_then(TrieNode::value))
        }
        _ => Err(())
    }
}

fn subtrie_size<'a, K, V>(node: &'a TrieNode<K, V>) -> usize {
    let mut size = if node.key_value.is_some() { 1 } else { 0 };

    for child in &node.children {
        if let &Some(ref child) = child {
            size += subtrie_size(&child);
        }
    }

    size
}

impl <'a, K, V> SubTrieMut<'a, K, V> where K: TrieKey {
    /// Look up the value for the given key, which should be an extension of this subtrie's key.
    pub fn get(&self, key: &K) -> SubTrieResult<&V> {
        subtrie_get(&self.prefix, &*self.node, key)
    }

    /// Compute the size of this subtrie.
    ///
    /// This isn't a constant time operation and involves a full traversal of the subtrie.
    pub fn len(&self) -> usize {
        subtrie_size(&self.node)
    }

    /// Insert a value in this subtrie. The key should be an extension of this subtrie's key.
    pub fn insert(&mut self, key: K, value: V) -> SubTrieResult<V> {
        let key_enc = key.encode();
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

    generate_trie_node_methods!();
}

fn stripped(mut key: NibbleVec, prefix: &NibbleVec) -> NibbleVec {
    key.split(prefix.len())
}
