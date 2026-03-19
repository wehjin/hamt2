pub mod core;
pub mod mem;
pub mod space;

#[cfg(test)]
mod tests {
    use crate::client::QueryError;
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

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut trie = SpaceTrie::new();
        trie.deep_insert([4, 2], MemValue::U32(42)).unwrap();
        {
            let value = trie.deep_query_value([4]).unwrap();
            let Some(MemValue::MapBase(map_base)) = value else {
                panic!("expected map_base");
            };
            assert_eq!(1, map_base.map().len());
            assert_eq!(1, map_base.base().len());
        }
        {
            let value = trie.deep_query_value([4, 2]).unwrap();
            assert_eq!(Some(MemValue::U32(42)), value);
        }
        {
            let value = trie.deep_query_value([4, 1]).unwrap();
            assert_eq!(None, value);
        }
        {
            let value = trie.deep_query_value([5, 2]).unwrap();
            assert_eq!(None, value);
        }
        {
            let result = trie.deep_query_value([4, 2, -1]);
            let Err(QueryError::NoSubtrieAtKeyIndex(1)) = result else {
                panic!("expected NoSubtrieAtKeyIndex(1)")
            };
        }
    }
}
