use Trie;

#[test]
fn get_and_insert() {
    let mut trie = Trie::new();
    let data = [
        ("abcdefgh", 18),
        ("abc", 17),
        ("acbdef", 16),
        ("bcdefgh", 15)
    ];

    for &(key, val) in &data {
        trie.insert(key, val);
    }

    for &(key, val) in &data {
        assert_eq!(*trie.get_mut(&key).unwrap(), val);
    }

    assert_eq!(trie.get_mut(&"nonexistant"), None);
    assert_eq!(trie.get_mut(&"abcdef"), None);
}

#[test]
fn insert_replace() {
    let mut trie = Trie::new();
    assert_eq!(trie.insert("haskell", 18), None);
    assert_eq!(trie.insert("haskell", 36), Some(18));
}
