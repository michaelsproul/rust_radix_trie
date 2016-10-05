use {TrieNode, TrieKey, NibbleVec};
use keys::{match_keys, KeyMatch};

impl<K, V> TrieNode<K, V> where K: TrieKey {
    pub fn get(&self, nv: &NibbleVec) -> Option<&TrieNode<K, V>> {
        iterative_get(self, nv)
    }

    pub fn get_mut(&mut self, nv: &NibbleVec) -> Option<&mut TrieNode<K, V>> {
        iterative_get_mut(self, nv)
    }

    pub fn insert(&mut self, key: K, value: V, nv: NibbleVec) -> Option<V> {
        iterative_insert(self, key, value, nv)
    }

    pub fn get_ancestor(&self, nv: &NibbleVec) -> Option<&TrieNode<K, V>> {
        get_ancestor(self, nv)
    }
}

/// Identity macro to allow expansion of the "mutability" token tree.
#[macro_export]
macro_rules! id {
    ($e:item) => { $e }
}

macro_rules! get_func {
    (
        name: $name:ident,
        trie_type: $trie_type:ty,
        mutability: $($mut_:tt)*
    ) => {id!{
        fn $name<'a, K, V>(trie: $trie_type, nv: &NibbleVec) -> Option<$trie_type> {
            if nv.len() == 0 {
                return Some(trie);
            }

            let mut prev = trie;
            let mut depth = 0;

            loop {
                let bucket = nv.get(depth) as usize;
                let current = prev;
                if let Some(ref $($mut_)* child) = current.children[bucket] {
                    match match_keys(depth, nv, &child.key) {
                        KeyMatch::Full => {
                            return Some(child);
                        }
                        KeyMatch::SecondPrefix => {
                            depth += child.key.len();
                            prev = child;
                        }
                        _ => {
                            return None;
                        }
                    }
                } else {
                    return None;
                }
            }
        }
    }}
}

get_func!(name: iterative_get, trie_type: &'a TrieNode<K, V>, mutability: );
get_func!(name: iterative_get_mut, trie_type: &'a mut TrieNode<K, V>, mutability: mut);

fn iterative_insert<'a, K, V>(trie: &'a mut TrieNode<K, V>, key: K, value: V, mut nv: NibbleVec)
    -> Option<V> where K: TrieKey
{
    if nv.len() == 0 {
        return trie.replace_value(key, value);
    }

    let mut prev = trie;
    let mut depth = 0;

    loop {
        let bucket = nv.get(depth) as usize;
        let current = prev;
        if let Some(ref mut child) = current.children[bucket] {
            match match_keys(depth, &nv, &child.key) {
                KeyMatch::Full => {
                    return child.replace_value(key, value);
                }
                KeyMatch::Partial(idx) => {
                    // Split the existing child.
                    child.split(idx);

                    // Insert the new key below the prefix node.
                    let new_key = nv.split(depth + idx);
                    let new_key_bucket = new_key.get(0) as usize;

                    child.add_child(
                        new_key_bucket,
                        Box::new(TrieNode::with_key_value(new_key, key, value))
                    );

                    return None;
                }
                KeyMatch::FirstPrefix => {
                    child.split(nv.len() - depth);
                    child.add_key_value(key, value);
                    return None;
                }
                KeyMatch::SecondPrefix => {
                    depth += child.key.len();
                    prev = child;
                }
            }
        } else {
            let node_key = nv.split(depth);
            current.add_child(bucket, Box::new(TrieNode::with_key_value(node_key, key, value)));
            return None;
        }
    }
}

// TODO: Mutable ancestor will be hard.
fn get_ancestor<'a, K, V>(trie: &'a TrieNode<K, V>, nv: &NibbleVec) -> Option<&'a TrieNode<K, V>>
    where K: TrieKey
{
    if nv.len() == 0 {
        return trie.as_value_node();
    }

    let mut prev = trie;
    // The ancestor is such that all nodes upto and including `prev` have
    // already been considered.
    let mut ancestor = prev.as_value_node();
    let mut depth = 0;

    loop {
        let bucket = nv.get(depth) as usize;
        let current = prev;
        if let Some(ref child) = current.children[bucket] {
            match match_keys(depth, &nv, &child.key) {
                KeyMatch::Full => {
                    return child.as_value_node().or(ancestor);
                }
                KeyMatch::FirstPrefix | KeyMatch::Partial(_) => {
                    return ancestor;
                }
                KeyMatch::SecondPrefix => {
                    depth += child.key.len();
                    ancestor = child.as_value_node().or(ancestor);
                    prev = child;
                }
            }
        } else {
            return ancestor;
        }
    }
}
