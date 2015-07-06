use std::collections::HashSet;
use Trie;

const TEST_DATA: [(&'static str, u32); 7] = [
        ("abcdefgh", 19),
        ("abcdef", 18),
        ("abcd", 17),
        ("ab", 16),
        ("a", 15),
        ("acbdef", 30),
        ("bcdefgh", 29)
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
fn get_node_nonexistant() {
    let trie = test_trie();
    assert!(trie.get_node(&"nonexistant").is_none());
    assert!(trie.get_node(&"").is_some());
}

#[test]
fn get_node() {
    let mut trie = Trie::new();
    trie.insert("hello", 55);
    assert!(trie.get_node(&"h").is_none());
    assert!(trie.get_node(&"hello").is_some());
}
#[test]
fn get_node_string() {
    let mut trie = Trie::new();
    trie.insert("hello".to_string(), 55);

    let h = "h".to_string();
    let hello = "hello".to_string();

    assert!(trie.get_node(&h).is_none());
    assert!(trie.get_node(&hello).is_some());
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
fn remove() {
    let mut trie = test_trie();

    // Remove.
    for &(key, val) in &TEST_DATA {
        assert_eq!(trie.remove(&key), Some(val));
        assert!(trie.check_integrity());
    }

    // Check non-existance.
    for &(key, _) in &TEST_DATA {
        assert!(trie.get(&key).is_none());
    }
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
fn iter() {
    type Set = HashSet<(&'static str, u32)>;
    let trie = test_trie();
    let expected = TEST_DATA.iter().map(|&x| x).collect::<Set>();
    let observed = trie.iter().map(|(&k, &v)| (k, v)).collect::<Set>();
    assert_eq!(expected, observed);
}

#[test]
fn get_descendant() {
    let trie = test_trie();
    assert_eq!(trie.get_descendant(&"abcdefgh").and_then(|t| t.value()), Some(&19));
    assert_eq!(trie.get_descendant(&"abcdefg").and_then(|t| t.value()), Some(&19));
    assert!(trie.get_descendant(&"acbg").is_none());
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
