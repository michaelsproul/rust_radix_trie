extern crate radix_trie;

use std::fmt::Debug;

use radix_trie::{Trie, NibbleVec, TrieKey};
use radix_trie::traversal::RefTraversal;

struct GetChildren;

impl<'a, K: Debug + TrieKey + 'a> RefTraversal<'a, K, u32> for GetChildren {
    type Input = &'a Trie<K, u32>;
    type Output = u32;

    fn default_result() -> Self::Output { 0u32 }

    fn child_match_fn(trie: &'a Trie<K, u32>, input: Self::Input, nv: NibbleVec) -> Self::Output {
        println!("children {:?}, {:?}, {:?}", trie.key(), trie.value(), nv);
        // Children code goes here
        for key in trie.keys() {
            // println!("{:?}", key);
            match input.get_node(&key) {
                Some(k) => {
                    Self::run(&input, &k, NibbleVec::from_byte_vec(key.encode()));
                }
                None => { }
            }
        }
        trie.value().unwrap_or(&0u32).clone()
    }

    fn action_fn(trie: &'a Trie<K, u32>, intermediate: Self::Output, bucket: usize) -> Self::Output {
        intermediate + trie.value().unwrap_or(&0u32)
    }
}


fn main() {
    let mut t = Trie::new();
    t.insert("aba", 5);
    t.insert("abb", 7);
    t.insert("abc", 9);

    println!("{:#?}", t);

   // println!("Ancestor\n{:#?}", t.get_ancestor(&"aba"));
    println!("Raw ancestor\n{:#?}", t.get_raw_ancestor(&"aba"));


//    let sum = GetChildren::run(&t, &t, NibbleVec::from_byte_vec("ab".encode()));

  //  println!("{}", sum);
}
