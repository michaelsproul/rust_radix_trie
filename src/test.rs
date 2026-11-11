use crate::keys::TrieKey;
use crate::{NibbleVec, Trie, TrieCommon};
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::collections::{BTreeMap, HashSet};
use std::iter::{FromIterator, FusedIterator};

const TEST_DATA: [(&'static str, u32); 7] = [
    ("abcdefgh", 19),
    ("abcdef", 18),
    ("abcd", 17),
    ("ab", 16),
    ("a", 15),
    ("acbdef", 30),
    ("bcdefgh", 29),
];

fn test_trie() -> Trie<&'static str, u32> {
    let mut trie = Trie::new();

    for &(key, val) in &TEST_DATA {
        trie.insert(key, val);
        assert!(trie.check_integrity());
    }

    trie
}

#[test]
fn get_nonexistant() {
    let trie = test_trie();
    assert!(trie.get(&"nonexistant").is_none());
    assert!(trie.get(&"").is_none());
}

#[test]
fn subtrie_nonexistant() {
    let trie = test_trie();
    assert!(trie.subtrie(&"nonexistant").is_none());
    assert!(trie.subtrie(&"").is_some());
}

#[test]
fn unicode() {
    let mut trie = Trie::new();
    trie.insert("bär", 1);
    trie.insert("bären", 2);

    assert_eq!(*trie.get("bär").unwrap(), 1);
    let values = trie
        .get_raw_descendant("bä")
        .unwrap()
        .values()
        .collect::<HashSet<_>>();
    assert_eq!([1, 2].iter().collect::<HashSet<_>>(), values);
}

#[test]
fn subtrie() {
    let mut trie = Trie::new();
    trie.insert("hello", 55);
    assert!(trie.subtrie(&"h").is_none());
    assert!(trie.subtrie(&"hello").is_some());
}
#[test]
fn subtrie_string() {
    let mut trie = Trie::new();
    trie.insert("hello".to_string(), 55);

    let h = "h".to_string();
    let hello = "hello".to_string();

    assert!(trie.subtrie(&h).is_none());
    assert!(trie.subtrie(&hello).is_some());
}

#[test]
fn empty_key() {
    let mut trie = test_trie();
    trie.insert(&"", 99);
    assert_eq!(*trie.get(&"").unwrap(), 99);
    assert_eq!(trie.remove(&""), Some(99));
}

#[test]
fn insert() {
    let trie = test_trie();

    for &(key, val) in &TEST_DATA {
        assert_eq!(*trie.get(&key).unwrap(), val);
    }

    assert!(trie.check_integrity());
    assert_eq!(trie.len(), TEST_DATA.len());
}

#[test]
fn insert_replace() {
    let mut trie = Trie::new();
    assert_eq!(trie.insert("haskell", 18), None);
    let length = trie.len();
    assert_eq!(trie.insert("haskell", 36), Some(18));
    assert_eq!(trie.len(), length);
}

#[test]
fn map_with_default() {
    let mut trie = test_trie();
    trie.map_with_default(&"abcd", |x| *x = *x + 1, 42);
    assert_eq!(*trie.get(&"abcd").unwrap(), 17 + 1);
    trie.map_with_default(&"zzz", |x| *x = *x + 1, 42);
    assert_eq!(*trie.get(&"zzz").unwrap(), 42);
}

#[test]
fn remove() {
    let mut trie = test_trie();

    // Remove.
    for &(key, val) in &TEST_DATA {
        let res = trie.remove(&key);
        assert_eq!(res, Some(val));
        assert!(trie.check_integrity());
    }

    // Check non-existance.
    for &(key, _) in &TEST_DATA {
        assert!(trie.get(&key).is_none());
    }
}

#[test]
fn remove_simple() {
    let mut trie = Trie::new();

    trie.insert("HELL", 66);
    trie.insert("HELLO", 77);
    let val = trie.remove(&"HELLO");
    assert_eq!(val, Some(77));
}

#[test]
fn remove_non_existent() {
    let mut trie = Trie::new();

    trie.insert("acab", true);

    assert_eq!(trie.remove(&"abc"), None);
    assert_eq!(trie.remove(&"acaba"), None);
    assert_eq!(trie.remove(&"a"), None);
    assert_eq!(trie.remove(&""), None);
    assert_eq!(trie.len(), 1);

    trie.insert("acaz", true);

    assert_eq!(trie.remove(&"acb"), None);
    assert_eq!(trie.remove(&"acaca"), None);
    assert_eq!(trie.remove(&"aca"), None);
    assert_eq!(trie.len(), 2);
}

#[test]
fn nearest_ancestor_root() {
    let mut trie = Trie::new();
    trie.insert("", 55);
    assert_eq!(trie.get_ancestor_value(&""), Some(&55));
}

#[test]
fn nearest_ancestor() {
    let trie = test_trie();
    assert_eq!(trie.get_ancestor_value(&""), None);

    // Test identity prefixes.
    for &(key, val) in &TEST_DATA {
        assert_eq!(trie.get_ancestor_value(&key), Some(&val));
    }

    assert_eq!(trie.get_ancestor_value(&"abcdefg"), trie.get(&"abcdef"));
    assert_eq!(trie.get_ancestor_value(&"abcde"), trie.get(&"abcd"));
    assert_eq!(trie.get_ancestor_value(&"aauksdjk"), trie.get(&"a"));
}

#[test]
fn nearest_ancestor_no_child_fn() {
    let mut t = Trie::new();
    t.insert("ab", 5);
    let anc = t.get_ancestor(&"abc").unwrap();
    assert_eq!(*anc.value().unwrap(), 5);
}

#[test]
fn raw_ancestor() {
    let mut t = Trie::new();

    for &(key, _) in &TEST_DATA {
        assert_eq!(t.get_raw_ancestor(&key).key(), t.key());
    }

    t.insert("wow", 0);
    t.insert("hella", 1);
    t.insert("hellb", 2);

    // Ancestor should be "hell" node.
    let anc = t.get_raw_ancestor(&"hello");
    assert_eq!(anc.len(), 2);
}

// Check that the subtrie prefix is correct for raw_ancestor.
#[test]
fn raw_ancestor_prefix() {
    let mut t = Trie::new();

    t.insert("abac", ());
    t.insert("abaz", ());

    let anc = t.get_raw_ancestor(&"aba");
    assert_eq!(anc.prefix, "aba".encode());
}

#[test]
fn iter() {
    type Set = HashSet<(&'static str, u32)>;
    let trie = test_trie();
    let expected = TEST_DATA.iter().map(|&x| x).collect::<Set>();
    let observed = trie.iter().map(|(&k, &v)| (k, v)).collect::<Set>();
    assert_eq!(expected, observed);
}

#[test]
fn get_raw_descendant() {
    let trie = test_trie();
    assert_eq!(
        trie.get_raw_descendant(&"abcdefgh").and_then(|t| t.value()),
        Some(&19)
    );
    assert_eq!(
        trie.get_raw_descendant(&"abcdefg").and_then(|t| t.value()),
        Some(&19)
    );
    assert!(trie.get_raw_descendant(&"acbg").is_none());
}

#[test]
fn raw_descendant_prefix() {
    let mut t = Trie::new();

    t.insert("abczzzz", ());
    t.insert("abcaaaa", ());

    assert_eq!(t.get_raw_descendant(&"a").unwrap().prefix, "abc".encode());
    assert_eq!(t.get_raw_descendant(&"abc").unwrap().prefix, "abc".encode());
    assert_eq!(
        t.get_raw_descendant(&"abca").unwrap().prefix,
        "abcaaaa".encode()
    );
}

#[test]
fn get_prefix_bug() {
    let mut trie = Trie::new();
    trie.insert("abdc", 5);
    trie.insert("abde", 6);
    assert!(trie.get(&"abc").is_none());
}

#[test]
fn get_ancestor_bug() {
    let mut trie = Trie::new();
    trie.insert("abc", 1);
    trie.insert("abcde", 2);
    assert_eq!(trie.get_ancestor_value(&"abcdz"), Some(&1));
}

#[test]
fn root_replace_bug() {
    let mut trie = Trie::new();
    trie.insert("a", ());
    trie.insert("p", ());
    trie.remove(&"a");
    assert_eq!(trie.len(), 1);
    trie.remove(&"p");
    assert_eq!(trie.len(), 0);
}

#[test]
fn subtrie_insert() {
    let mut trie = Trie::new();
    trie.insert("abc", 3);
    {
        let mut subtrie = trie.subtrie_mut(&"abc").unwrap();
        assert_eq!(subtrie.insert("somerandomshit", 666), Err(()));
        assert_eq!(subtrie.insert("abcdef", 6), Ok(None));
        assert_eq!(subtrie.insert("abc", 9), Ok(Some(3)));
    }
    assert_eq!(trie.get(&"abc"), Some(&9));
    assert_eq!(trie.get(&"abcdef"), Some(&6));
    assert_eq!(trie.len(), 2);
}

#[test]
fn subtrie_len() {
    let trie = test_trie();
    assert_eq!(trie.subtrie(&"ab").unwrap().len(), 4);
    assert_eq!(trie.subtrie(&"a").unwrap().len(), 6);
    assert_eq!(trie.subtrie(&"").unwrap().len(), trie.len());
    assert_eq!(trie.subtrie(&"bcdefgh").unwrap().len(), 1);
}

// Subtrie functions that return references should return references valid for
// the lifetime of the *original trie* from which they were borrowed, NOT
// just the lifetime of the subtrie (which may be shorter).
#[test]
fn subtrie_lifetime() {
    let trie = test_trie();
    let subtrie_value = {
        let subtrie = trie.subtrie(&"ab").unwrap();
        subtrie.value()
    };
    assert_eq!(*subtrie_value.unwrap(), 16);
}

#[test]
fn subtrie_mut_lifetime() {
    let mut trie = test_trie();
    let subtrie_value = {
        let mut subtrie = trie.subtrie_mut(&"ab").unwrap();
        *subtrie.value_mut().unwrap() = 999;
        subtrie.value()
    };
    assert_eq!(*subtrie_value.unwrap(), 999);
}

#[test]
fn ancestor_key() {
    let trie = test_trie();
    let subtrie = trie.get_ancestor(&"abcde").unwrap();
    assert_eq!(*subtrie.key().unwrap(), "abcd");
    assert_eq!(subtrie.prefix, "abcd".encode());
    assert_eq!(*subtrie.get(&"abcdef").unwrap().unwrap(), 18);
    assert_eq!(*subtrie.get(&"abcdefgh").unwrap().unwrap(), 19);
}

#[test]
fn child_subtrie_keys() {
    let trie = test_trie();
    let subtrie = trie.subtrie(&"abcd").unwrap();
    for subsubtrie in subtrie.children() {
        // This subtrie should be for "abcde".
        assert_eq!(*subsubtrie.get(&"abcdef").unwrap().unwrap(), 18);
        assert_eq!(*subsubtrie.get(&"abcdefgh").unwrap().unwrap(), 19);
    }
}

#[test]
fn int_keys() {
    let mut trie = Trie::new();
    trie.insert(0x00ffu64, "asdf");
    trie.insert(0xdeadbeefu64, "asdf");
    assert!(trie.check_integrity());
}

#[test]
fn from_iter() {
    let trie: Trie<&str, u32> = Trie::from_iter(vec![("test", 10), ("hello", 12)]);
    assert_eq!(*trie.get(&"test").unwrap(), 10);
    assert_eq!(*trie.get(&"hello").unwrap(), 12);
    assert_eq!(trie.len(), 2);
}

#[test]
fn test_get_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    assert_eq!(*trie.get("/boot").unwrap(), "dir");
}

#[test]
fn test_get_mut_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    assert_eq!(*trie.get_mut("/boot").unwrap(), "dir");
}

#[test]
fn test_remove_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    assert_eq!(trie.remove("/boot").unwrap(), "dir");
}

#[test]
fn test_subtrie_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    trie.insert("/boot/lol".to_string(), "dir");
    trie.insert("/bleh".to_string(), "ohi");
    let subtrie = trie.subtrie("/boot").unwrap();
    assert_eq!(*subtrie.value().unwrap(), "dir");
}

#[test]
fn test_subtrie_mut_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    trie.insert("/boot/lol".to_string(), "dir");
    trie.insert("/bleh".to_string(), "ohi");
    let subtrie = trie.subtrie_mut("/boot").unwrap();
    assert_eq!(*subtrie.value().unwrap(), "dir");
}

#[test]
fn test_get_ancestor_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "ancestor");
    trie.insert("/boot/lol".to_string(), "dir");
    trie.insert("/bleh".to_string(), "ohi");
    let subtrie = trie.get_ancestor("/boot/lo").unwrap();
    assert_eq!(*subtrie.value().unwrap(), "ancestor");
}

#[test]
fn test_get_ancestor_value_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "ancestor");
    trie.insert("/boot/lol".to_string(), "dir");
    trie.insert("/bleh".to_string(), "ohi");
    let ancestor_val = trie.get_ancestor_value("/boot/lo").unwrap();
    assert_eq!(*ancestor_val, "ancestor");
}

#[test]
fn test_get_raw_ancestor_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "boot");
    trie.insert("/boot/lol".to_string(), "dir");
    trie.insert("/bleh".to_string(), "ohi");
    let subtrie = trie.get_raw_ancestor("/boot/lo");
    assert_eq!(*subtrie.value().unwrap(), "boot");
}

#[test]
fn test_get_raw_descendant_borrow() {
    let mut trie = Trie::new();
    trie.insert("/boot".to_string(), "dir");
    trie.insert("/boot/lol".to_string(), "lol");
    trie.insert("/bleh".to_string(), "ohi");
    let subtrie = trie.get_raw_descendant("/boot/lo").unwrap();
    assert_eq!(*subtrie.value().unwrap(), "lol");
}

#[test]
fn test_prefix() {
    let mut t = Trie::<u8, ()>::new();
    t.insert(0xf1, ());
    t.remove(&0xf2);
    t.insert(0xf2, ());
    println!("{:#?}", t);
    assert_eq!(t.prefix(), [].as_ref());
    let first = t.children().next().unwrap();
    assert_eq!(first.prefix(), [0xf].as_ref());
    let mut c = first.children();
    let second = c.next().unwrap();
    let third = c.next().unwrap();
    assert!(c.next().is_none());
    assert_eq!(second.prefix(), [0x1].as_ref());
    assert_eq!(third.prefix(), [0x2].as_ref());
}

#[test]
fn clone() {
    let mut t1 = test_trie();
    let mut t2 = t1.clone();

    assert_eq!(t1, t2);
    t1.insert("abc", 22);

    assert_ne!(t1, t2);
    t2.insert("abc", 22);

    assert_eq!(t1, t2);
}

/// Check value order and that repeated calls after exhaustion keep returning `None`.
fn assert_fused_values<'a>(mut iter: impl FusedIterator<Item = &'a usize>, expected: &[usize]) {
    for value in expected {
        assert_eq!(iter.next(), Some(value));
    }
    for _ in 0..3 {
        assert_eq!(iter.next(), None);
    }
}

/// Check that generic and byte-specialized ancestor iterators yield the same expected values.
fn assert_values(trie: &Trie<Vec<u8>, usize>, query: &[u8], expected: &[usize]) {
    assert_fused_values(trie.get_all_ancestor_values(query), expected);
    assert_fused_values(trie.get_all_ancestor_values_bytes(query), expected);
}

/// Compare both ancestor iterators with a reference map filtered by `starts_with`,
/// ordering matching keys from shortest to longest.
fn assert_oracle(trie: &Trie<Vec<u8>, usize>, map: &BTreeMap<Vec<u8>, usize>, query: &[u8]) {
    let mut matches: Vec<_> = map
        .iter()
        .filter(|(prefix, _)| query.starts_with(prefix))
        .collect();
    matches.sort_by_key(|(prefix, _)| prefix.len());
    let expected: Vec<_> = matches.into_iter().map(|(_, value)| *value).collect();
    assert_values(trie, query, &expected);
}

/// Generate every sequence over `alphabet` up to `max_len`, including the empty
/// sequence, in increasing length and then alphabet iteration order.
fn sequences(alphabet: &[u8], max_len: usize) -> Vec<Vec<u8>> {
    let mut result = vec![vec![]];
    let mut level = vec![vec![]];
    for _ in 0..max_len {
        level = level
            .iter()
            .flat_map(|prefix| {
                alphabet.iter().map(move |byte| {
                    let mut key = prefix.clone();
                    key.push(*byte);
                    key
                })
            })
            .collect();
        result.extend(level.iter().cloned());
    }
    result
}

/// Check empty tries and queries, empty-key values, exact matches, and nested
/// prefixes yielded shortest first regardless of insertion order.
#[test]
fn empty_queries_and_nested_prefixes_are_shortest_first() {
    let mut trie = Trie::new();
    assert_values(&trie, b"", &[]);
    assert_values(&trie, b"abc", &[]);

    // Insert longest first to ensure ordering is independent of insertion.
    for (key, value) in [(b"abc".as_slice(), 3), (b"ab", 2), (b"a", 1)] {
        trie.insert(key.to_vec(), value);
    }
    assert_values(&trie, b"", &[]);
    assert_values(&trie, b"abc", &[1, 2, 3]);
    assert_values(&trie, b"abcd", &[1, 2, 3]);
    assert_values(&trie, b"ac", &[1]);
    assert_values(&trie, b"z", &[]);

    trie.insert(vec![], 0);
    assert_values(&trie, b"", &[0]);
    assert_values(&trie, b"abc", &[0, 1, 2, 3]);
    assert_values(&trie, b"z", &[0]);
    assert_eq!(trie.get_ancestor_value(b"abcd".as_slice()), Some(&3));
}

/// Ensure branching nodes without values are skipped and queries ending or
/// diverging within compressed edges retain only previously matched ancestors.
#[test]
fn valueless_ancestors_and_partial_compressed_edges() {
    let mut trie = Trie::new();
    for (key, value) in [(b"a".as_slice(), 1), (b"abcdef", 2), (b"abcxyz", 3)] {
        trie.insert(key.to_vec(), value);
    }
    // "abc" is a branching node, not a stored prefix.
    assert_values(&trie, b"abc", &[1]);
    assert_values(&trie, b"abcde", &[1]);
    assert_values(&trie, b"abcdex", &[1]);
    assert_values(&trie, b"abcdef", &[1, 2]);
    assert_values(&trie, b"abcxyz!", &[1, 3]);
    assert_values(&trie, b"b", &[]);
    assert!(trie.check_integrity());
}

/// Exhaust all one- and two-byte queries against binary keys whose shared high
/// nibbles force odd-length edges, including NUL and non-UTF-8 bytes.
#[test]
fn all_byte_values_and_odd_nibble_edges() {
    let mut trie = Trie::new();
    trie.insert(vec![], 0);
    for byte in 0..=255u8 {
        let value = 1 + usize::from(byte) * 3;
        trie.insert(vec![byte], value);
        trie.insert(vec![byte, 0], value + 1);
        trie.insert(vec![byte, 255], value + 2);
    }
    // Adjacent bytes share their high nibble, creating odd-length edges.
    // Exhaust every two-byte query, including NUL and non-UTF-8 bytes.
    for first in 0..=255u8 {
        let value = 1 + usize::from(first) * 3;
        assert_values(&trie, &[first], &[0, value]);
        for second in 0..=255u8 {
            let mut expected = vec![0, value];
            match second {
                0 => expected.push(value + 1),
                255 => expected.push(value + 2),
                _ => (),
            }
            assert_values(&trie, &[first, second], &expected);
        }
    }
    assert!(trie.check_integrity());
}

/// Compare compressed-edge matching with the reference model across even and odd
/// nibble alignments, short and long labels, query mutations, and edge merging.
#[test]
fn all_compressed_edge_alignments_match_starts_with() {
    let flip = |key: &mut [u8], nibble: usize, delta: u8| {
        key[nibble / 2] ^= delta << (if nibble % 2 == 0 { 4 } else { 0 });
    };
    for depth in 0usize..8 {
        for label_len in (1..=16).chain([127, 128, 129, 1023, 1024, 1025]) {
            let end = depth + label_len;
            let key: Vec<_> = (0..end.div_ceil(2) + 3)
                .map(|i| (i * 37 + 0xab) as u8)
                .collect();
            let mut parent_sibling = key.clone();
            flip(&mut parent_sibling, depth, 1);
            let mut edge_sibling = key.clone();
            flip(&mut edge_sibling, end, 1);
            let mut map: BTreeMap<_, _> = [
                (vec![], 0),
                (key.clone(), 1),
                (parent_sibling.clone(), 2),
                (edge_sibling.clone(), 3),
            ]
            .into_iter()
            .collect();
            let mut trie: Trie<_, _> = map.clone().into_iter().collect();
            // Branches at `depth` and `end` force an internal compressed edge
            // with exactly the requested start alignment and label length.
            let edge = trie.get_raw_ancestor(&key[..end.div_ceil(2)]);
            assert_eq!(edge.prefix().len(), label_len);
            assert!(!edge.is_leaf());
            assert_eq!(edge.value(), None);
            for query in map.keys() {
                assert_oracle(&trie, &map, query);
            }
            for length in 0..=key.len() {
                assert_oracle(&trie, &map, &key[..length]);
            }
            let positions: Vec<_> = if label_len <= 16 {
                (0..key.len() * 2).collect()
            } else {
                vec![
                    0,
                    depth,
                    depth + label_len / 2,
                    end - 1,
                    end,
                    key.len() * 2 - 1,
                ]
            };
            // Exhaust every replacement nibble for short labels, including
            // nibbles just outside the edge (especially odd-label padding).
            for position in positions {
                for delta in 0..16 {
                    let mut query = key.clone();
                    flip(&mut query, position, delta);
                    assert_oracle(&trie, &map, &query);
                }
            }
            // Merging changes internal edges into leaves and can change the
            // alignment of the remaining edge without changing any keys.
            for removed in [&edge_sibling, &parent_sibling] {
                assert_eq!(trie.remove(removed), map.remove(removed));
                assert!(trie.check_integrity());
                assert_oracle(&trie, &map, &key);
                assert_oracle(&trie, &map, removed);
            }
        }
    }
}

/// Exercise the odd-depth leaf suffix optimization with changed boundary bytes,
/// truncated queries, extended queries, and mismatches at the end of a key.
#[test]
fn odd_depth_leaf_suffixes_match_starts_with() {
    for depth in (1usize..16).step_by(2) {
        for suffix_len in [1, 2, 3, 31, 128, 1024] {
            let key: Vec<_> = (0..depth / 2 + suffix_len)
                .map(|i| (i * 53 + 0x1f) as u8)
                .collect();
            let mut sibling = key.clone();
            sibling[depth / 2] ^= 1;
            let map: BTreeMap<_, _> = [(vec![], 0), (key.clone(), 1), (sibling, 2)]
                .into_iter()
                .collect();
            let trie: Trie<_, _> = map.clone().into_iter().collect();
            let leaf = trie.subtrie(&key).unwrap();
            assert!(leaf.is_leaf());
            assert_eq!(leaf.prefix().len(), key.len() * 2 - depth);
            for byte in 0..=255 {
                let mut query = key.clone();
                query[depth / 2] = byte;
                assert_oracle(&trie, &map, &query);
            }
            for length in 0..=key.len() {
                assert_oracle(&trie, &map, &key[..length]);
            }
            let mut query = key.clone();
            query.push(255);
            assert_oracle(&trie, &map, &query);
            *query.last_mut().unwrap() = 0;
            query[key.len() - 1] ^= 1;
            assert_oracle(&trie, &map, &query);
        }
    }
}

/// Check long compressed edges and deeply nested stored prefixes, including
/// queries that end early, diverge at the final nibble, or extend past a leaf.
#[test]
fn long_edges_and_deep_nesting() {
    let key: Vec<_> = (0..8192).map(|i| (i % 256) as u8).collect();
    let mut trie = Trie::new();
    trie.insert(vec![], 0);
    trie.insert(key.clone(), 1);
    assert_values(&trie, &key, &[0, 1]);
    assert_values(&trie, &key[..key.len() - 1], &[0]);
    let mut query = key.clone();
    // Same high nibble, different low nibble at the end of a long edge.
    *query.last_mut().unwrap() ^= 1;
    assert_values(&trie, &query, &[0]);
    query = key.clone();
    query.push(0);
    assert_values(&trie, &query, &[0, 1]);

    let mut nested = Trie::new();
    for length in (0..=256).rev() {
        nested.insert(key[..length].to_vec(), length);
    }
    let expected: Vec<_> = (0..=256).collect();
    assert_values(&nested, &key, &expected);
    assert!(nested.check_integrity());
}

/// Check compatible string and byte query encodings, ownership of a temporary
/// encoded query, and returned byte-query values that outlive the query buffer.
#[test]
fn generic_queries_use_compatible_encodings_and_own_the_query() {
    let trie: Trie<String, usize> = [("".into(), 0), ("bär".into(), 1), ("bären".into(), 2)]
        .into_iter()
        .collect();
    assert_fused_values(trie.get_all_ancestor_values("bären!"), &[0, 1, 2]);
    // Prefix queries require compatible encodings, not a Borrow<Q> impl.
    assert_fused_values(
        trie.get_all_ancestor_values("bären!".as_bytes()),
        &[0, 1, 2],
    );
    let iter = trie.get_all_ancestor_values(&String::from("bären!"));
    assert_fused_values(iter, &[0, 1, 2]);

    let bytes: Trie<Vec<u8>, usize> = [(vec![], 0), (vec![255], 1)].into_iter().collect();
    let values = {
        let query = vec![255, 0];
        bytes
            .get_all_ancestor_values_bytes(&query)
            .collect::<Vec<_>>()
    };
    assert_eq!(values, [&0, &1]);
}

/// Ensure both iterators borrow non-`Clone` values and preserve equal values
/// stored under distinct matching keys.
#[test]
fn values_need_not_be_clone_and_duplicates_are_preserved() {
    /// Non-`Clone` payload used to verify iteration imposes no cloning requirement.
    struct Value(usize);
    let trie: Trie<Vec<u8>, Value> = [(vec![], Value(7)), (vec![0], Value(7))]
        .into_iter()
        .collect();
    assert_eq!(
        trie.get_all_ancestor_values_bytes(&[0, 1])
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [7, 7]
    );
    assert_eq!(
        trie.get_all_ancestor_values([0, 1].as_slice())
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        [7, 7]
    );
}

/// Test key whose elements encode as individual nibbles rather than full bytes,
/// allowing stored keys and queries with odd nibble lengths.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Nibbles(Vec<u8>);

impl TrieKey for Nibbles {
    /// Encode each element as one nibble without adding byte-alignment padding.
    fn encode(&self) -> NibbleVec<[u8; 64]> {
        let mut key = NibbleVec::new();
        for nibble in &self.0 {
            key.push(*nibble);
        }
        key
    }
}

/// Compare generic ancestor matching with a nibble-sequence reference model to
/// ensure odd-length encodings are matched without rounding to whole bytes.
#[test]
fn generic_encoding_preserves_half_byte_precision() {
    let queries = sequences(&[0, 1, 15], 4);
    let keys: Vec<_> = queries.iter().step_by(3).cloned().map(Nibbles).collect();
    let trie: Trie<_, _> = keys
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, k)| (k, i))
        .collect();
    for query in queries {
        let mut expected: Vec<_> = keys
            .iter()
            .enumerate()
            .filter(|(_, key)| query.starts_with(&key.0))
            .collect();
        expected.sort_by_key(|(_, key)| key.0.len());
        let expected: Vec<_> = expected.into_iter().map(|(i, _)| i).collect();
        assert_fused_values(trie.get_all_ancestor_values(&Nibbles(query)), &expected);
    }
    assert!(trie.check_integrity());
}

/// Exhaust small key subsets and queries, checking the reference model before
/// and after removals that exercise absent keys, valueless nodes, and edge merges.
#[test]
fn exhaustive_subsets_and_removals_match_starts_with() {
    let keys = sequences(&[0x10, 0x1f], 2);
    let queries = sequences(&[0x10, 0x1f, 0xff], 3);
    for subset in 0..1usize << keys.len() {
        let mut trie = Trie::new();
        let mut map = BTreeMap::new();
        for (i, key) in keys.iter().enumerate().rev() {
            if subset & (1 << i) != 0 {
                trie.insert(key.clone(), i);
                map.insert(key.clone(), i);
            }
        }
        for query in &queries {
            assert_oracle(&trie, &map, query);
        }
        // Remove present keys, absent keys and valueless internal prefixes.
        // This also exercises edge merging at both odd and even nibble depths.
        for key in &keys {
            assert_eq!(trie.remove(key), map.remove(key));
            assert_eq!(trie.len(), map.len());
            assert!(trie.check_integrity());
            for query in &queries {
                assert_oracle(&trie, &map, query);
            }
        }
        assert!(trie.is_empty());
    }
}

/// Use a fixed random seed to compare ancestor results with a reference map
/// through insertions, removals, value updates, and varied derived queries.
#[test]
fn seeded_random_mutations_match_starts_with() {
    let mut rng = StdRng::seed_from_u64(0x7072_6566_6978);
    let mut keys = vec![vec![]];
    for _ in 0..255 {
        let mut key = keys[rng.gen_range(0..keys.len())].clone();
        key.extend((0..rng.gen_range(1..=16)).map(|_| rng.r#gen::<u8>()));
        keys.push(key);
    }
    let mut trie = Trie::new();
    let mut map = BTreeMap::new();
    for value in 0..2048 {
        let key = &keys[rng.gen_range(0..keys.len())];
        match rng.gen_range(0..4) {
            0 => assert_eq!(trie.remove(key), map.remove(key)),
            1 => {
                if let Some(stored) = trie.get_mut(key) {
                    *stored = value;
                }
                if let Some(stored) = map.get_mut(key) {
                    *stored = value;
                }
            }
            _ => assert_eq!(
                trie.insert(key.clone(), value),
                map.insert(key.clone(), value)
            ),
        }
        assert_eq!(trie.len(), map.len());
        assert!(trie.check_integrity());
        assert_oracle(&trie, &map, key);
        for _ in 0..8 {
            let mut query = keys[rng.gen_range(0..keys.len())].clone();
            if rng.gen_bool(0.5) {
                query.truncate(rng.gen_range(0..=query.len()));
            } else {
                query.push(rng.r#gen());
            }
            assert_oracle(&trie, &map, &query);
        }
    }
}

/// Ensure deserialized byte-keyed tries preserve ancestor values and ordering
/// for empty keys, nested binary prefixes, and nonmatching suffixes.
#[cfg(feature = "serde")]
#[test]
fn deserialized_byte_keys_preserve_prefix_values() {
    use serde::Deserialize;
    use serde::de::value::{Error, MapDeserializer, SeqDeserializer};

    let map: BTreeMap<_, _> = [
        (vec![], 0),
        (vec![255], 1),
        (vec![255, 0], 2),
        (vec![255, 1], 3),
    ]
    .into_iter()
    .collect();
    let entries = map.iter().map(|(key, value)| {
        (
            SeqDeserializer::<_, Error>::new(key.iter().copied()),
            *value,
        )
    });
    let trie =
        Trie::<Vec<u8>, usize>::deserialize(MapDeserializer::<_, Error>::new(entries)).unwrap();
    for query in [
        vec![],
        vec![255],
        vec![255, 0, 1],
        vec![255, 1],
        vec![255, 2],
    ] {
        assert_oracle(&trie, &map, &query);
    }
    assert!(trie.check_integrity());
}
