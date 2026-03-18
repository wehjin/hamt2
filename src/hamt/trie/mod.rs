pub mod key;
pub mod map;
pub mod mem;
pub mod space;
pub mod value;

#[cfg(test)]
mod tests {
    use crate::hamt::trie::mem::value::MemValue;
    use crate::hamt::trie::space::SpaceTrie;

    #[tokio::test]
    async fn later_insertion_overwrites_earlier_insertion() {
        let trie = SpaceTrie::new()
            .insert(-1, MemValue::U32(42))
            .unwrap()
            .insert(-1, MemValue::U32(43))
            .unwrap();
        let value = trie.query_value(-1).unwrap();
        assert_eq!(Some(MemValue::U32(43)), value);
    }

    #[tokio::test]
    async fn different_keys_have_different_values() {
        let mut trie = SpaceTrie::new();
        // 33 keys will saturate the root block.
        let keys = (0..=32).collect::<Vec<_>>();
        for i in &keys {
            trie = trie.insert(*i, MemValue::U32(*i as u32)).unwrap();
        }
        let mut values = Vec::new();
        for i in &keys {
            let value = trie
                .query_value(*i)
                .expect(&format!("query for key: {}", i));
            values.push(value);
        }
        let expected = keys
            .iter()
            .map(|i| Some(MemValue::U32(*i as u32)))
            .collect::<Vec<_>>();
        assert_eq!(expected, values);
    }
}
