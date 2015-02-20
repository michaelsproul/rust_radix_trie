use Trie;

#[test]
fn get_and_insert() {
    let mut trie: Trie<&'static str, u32> = Trie::new();
    let data = [
        ("abcdefgh", 18),
        ("abc", 17),
        ("acbdef", 16),
        ("bcdefgh", 15)
    ];

    for &(key, val) in &data {
        trie.insert(key, val);
    }

    println!("{:?}", trie);

    for &(key, val) in &data {
        println!("Retrieving {:?}", key);
        assert_eq!(*trie.get_mut(&key).unwrap(), val);
    }
}
