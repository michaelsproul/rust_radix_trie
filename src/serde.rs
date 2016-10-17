extern crate serde;

use super::{Trie, TrieKey, TrieCommon};
use self::serde::{Serialize, Serializer, Deserialize, Deserializer, de, Error};
use std::marker::PhantomData;

impl<K, V> Serialize for Trie<K, V>
    where K: Serialize + TrieKey,
          V: Serialize
{
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        let mut state = try!(serializer.serialize_map(Some(self.len())));
        for (k, v) in self.iter() {
            try!(serializer.serialize_map_key(&mut state, k));
            try!(serializer.serialize_map_value(&mut state, v));
        }
        serializer.serialize_map_end(state)
    }
}


struct TrieVisitor<K, V> {
    marker: PhantomData<Trie<K, V>>,
}

impl<K, V> TrieVisitor<K, V> {
    fn new() -> Self {
        TrieVisitor { marker: PhantomData }
    }
}

impl<K, V> de::Visitor for TrieVisitor<K, V>
    where K: Deserialize + Clone + Eq + PartialEq + TrieKey,
          V: Deserialize
{
    type Value = Trie<K, V>;

    fn visit_map<M>(&mut self, mut visitor: M) -> Result<Self::Value, M::Error>
        where M: de::MapVisitor
    {
        let mut values = Trie::new();

        while let Some((key, value)) = try!(visitor.visit()) {
            values.insert(key, value);
        }

        try!(visitor.end());
        Ok(values)
    }

    fn visit_unit<E>(&mut self) -> Result<Self::Value, E>
        where E: Error
    {
        Ok(Trie::new())
    }
}

impl<K, V> Deserialize for Trie<K, V>
    where K: Deserialize + Clone + Eq + PartialEq + TrieKey,
          V: Deserialize
{
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_map(TrieVisitor::new())
    }
}

#[cfg(test)]
mod test {
    extern crate serde_test;
    use self::serde_test::{Token};
    use super::super::Trie;

    macro_rules! tests_de {
        ($($name:ident => $value:expr => $tokens:expr,)+) => {
            $(#[test]
            fn $name() {
                // Test ser/de roundtripping
                serde_test::assert_de_tokens(&$value, $tokens);
            })+
        }
    }

    macro_rules! tests_ser {
        ($($name:ident => $value:expr => $tokens:expr,)+) => {
            $(#[test]
            fn $name() {
                serde_test::assert_ser_tokens(&$value, $tokens);
            })+
        }
    }

    macro_rules! trie {
        () => {
            Trie::new()
        };
        ($($key:expr => $value:expr),+) => {
            {
                let mut map = Trie::new();
                $(map.insert($key, $value);)+
                map
            }
        }
    }

    tests_ser! {
        test_ser_empty_trie => Trie::<&str, isize>::new() => &[
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        test_ser_single_element_trie => trie!["1" => 2] => &[
            Token::MapStart(Some(1)),
            Token::MapSep,
            Token::Str("1"),
            Token::I32(2),
            Token::MapEnd,
        ],
        test_ser_multiple_element_trie => trie!["1" => 2, "3" => 4] => &[
            Token::MapStart(Some(2)),
            Token::MapSep,
            Token::Str("1"),
            Token::I32(2),

            Token::MapSep,
            Token::Str("3"),
            Token::I32(4),
            Token::MapEnd,
        ],
        test_ser_deep_trie => trie!["1" => trie![], "2" => trie!["3" => 4, "5" => 6]] => &[
            Token::MapStart(Some(2)),
            Token::MapSep,
            Token::Str("1"),
            Token::MapStart(Some(0)),
            Token::MapEnd,

            Token::MapSep,
            Token::Str("2"),
            Token::MapStart(Some(2)),
            Token::MapSep,
            Token::Str("3"),
            Token::I32(4),

            Token::MapSep,
            Token::Str("5"),
            Token::I32(6),
            Token::MapEnd,
            Token::MapEnd,
        ],
    }

    tests_de! {
        test_de_empty_trie1 => Trie::<String, isize>::new() => &[
            Token::Unit,
        ],
        test_de_empty_trie2 => Trie::<String, isize>::new() => &[
            Token::MapStart(Some(0)),
            Token::MapEnd,
        ],
        test_de_single_element_trie => trie!["1".to_string() => 2] => &[
            Token::MapStart(Some(1)),
                Token::MapSep,
                Token::Str("1"),
                Token::I32(2),
            Token::MapEnd,
        ],
        test_de_multiple_element_trie => trie!["1".to_string()  => 2, "3".to_string()  => 4] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::Str("1"),
                Token::I32(2),

                Token::MapSep,
                Token::Str("3"),
                Token::I32(4),
            Token::MapEnd,
        ],
        test_de_deep_trie => trie!["1".to_string()  => trie![], "2".to_string()  => trie!["3".to_string()  => 4, "5".to_string()  => 6]] => &[
            Token::MapStart(Some(2)),
                Token::MapSep,
                Token::Str("1"),
                Token::MapStart(Some(0)),
                Token::MapEnd,

                Token::MapSep,
                Token::Str("2"),
                Token::MapStart(Some(2)),
                    Token::MapSep,
                    Token::Str("3"),
                    Token::I32(4),

                    Token::MapSep,
                    Token::Str("5"),
                    Token::I32(6),
                Token::MapEnd,
            Token::MapEnd,
        ],
        test_de_empty_trie3 => Trie::<String, isize>::new() => &[
            Token::UnitStruct("Anything"),
        ],
        test_de_empty_trie4 => Trie::<String, isize>::new() => &[
            Token::StructStart("Anything", 0),
            Token::MapEnd,
        ],
    }
}
