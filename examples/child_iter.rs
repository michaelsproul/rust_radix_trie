extern crate radix_trie;

use radix_trie::Trie;

fn main() {
    let mut t = Trie::new();
    t.insert("a", 5);
    t.insert("b", 6);
    t.insert("c", 50);

    let sum = t.child_iter().fold(0, |acc, c| {
        println!("{:#?}", c);
        acc + *c.value().unwrap_or(&0)
    });
    println!("{}", sum);
}
