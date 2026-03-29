pub mod core;
pub mod mem;
pub mod space;

#[cfg(test)]
mod tests {
    use crate::space::file::FileSpace;
    use crate::space::mem::MemSpace;
    use crate::trie::mem::value::MemValue;
    use crate::trie::space::trie::SpaceTrie;

    #[test]
    fn file_trie_works() {
        let file = tempfile::NamedTempFile::new().expect("tempfile");
        {
            let mut space = FileSpace::new(file.path()).expect("create file space");
            let trie = SpaceTrie::connect(&space).expect("connect");
            trie.insert(1, MemValue::U32(1))
                .expect("insert")
                .commit(&mut space)
                .expect("commit");
        }
        {
            let space = FileSpace::load(file.path()).expect("load file space");
            let trie = SpaceTrie::connect(&space).expect("connect");
            let value = trie.query_value(1).expect("query_value");
            assert_eq!(Some(MemValue::U32(1)), value);
        }
    }

    #[tokio::test]
    async fn query_key_values_works() {
        let space = MemSpace::new();
        let mut trie = SpaceTrie::connect(&space).expect("connect");
        trie = trie.insert(1, MemValue::U32(1)).expect("insert");
        trie = trie.insert(2, MemValue::U32(2)).expect("insert");
        let key_values = trie.query_keys_values().expect("all_keys_values");
        let mut key_values = key_values
            .into_iter()
            .map(|kv| {
                let MemValue::U32(value) = kv.1 else {
                    panic!("expected U32");
                };
                (kv.0, value)
            })
            .collect::<Vec<_>>();
        key_values.sort();
        assert_eq!(vec![(1, 1), (2, 2)], key_values);
    }

    #[tokio::test]
    async fn multiple_commits_work() {
        let mut space = MemSpace::new();
        // Commit once.
        {
            let mut trie = SpaceTrie::connect(&space).unwrap();
            trie = trie.insert(-1, MemValue::U32(42)).unwrap();
            trie = trie.deep_insert([-2, 42], MemValue::U32(242)).unwrap();
            trie.commit(&mut space).unwrap();
        }
        // Commit again.
        {
            let mut trie = SpaceTrie::connect(&space).unwrap();
            trie = trie.insert(-1, MemValue::U32(84)).unwrap();
            trie.commit(&mut space).expect("commit");
        }
        // Query from both commits.
        {
            let trie = SpaceTrie::connect(&space).unwrap();
            assert_eq!(Some(MemValue::U32(84)), trie.query_value(-1).unwrap());
            assert_eq!(
                Some(MemValue::U32(242)),
                trie.deep_query_value([-2, 42]).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn persistence_works() {
        let mut space = MemSpace::new();
        // Commit some values.
        {
            let mut trie = SpaceTrie::connect(&space).unwrap();
            trie = trie.insert(-1, MemValue::U32(42)).unwrap();
            for a in 0..=32 {
                trie = trie.deep_insert([3, a], MemValue::U32(a as u32)).unwrap();
            }
            trie.commit(&mut space).unwrap();
        }
        // Test commited values.
        {
            let trie = SpaceTrie::connect(&space).unwrap();
            assert_eq!(Some(MemValue::U32(42)), trie.query_value(-1).unwrap());
            for a in 0..=32 {
                assert_eq!(
                    Some(MemValue::U32(a as u32)),
                    trie.deep_query_value([3, a]).unwrap()
                );
            }
        }
        // Deep insert values to saturate root blocks in deep tries.
        {
            let mut trie = SpaceTrie::connect(&space).unwrap();
            // Use at least 33 keys so that the root blook in the first trie is saturated.
            for i in 0..35 {
                let e = 5 + i;
                trie = trie.deep_insert([e, 0], MemValue::U32(e as u32)).unwrap();
            }
            // Use at least 33 keys so that the root block in the second trie is saturated.
            for i in 0..35 {
                let a = 3 + i;
                trie = trie.deep_insert([4, a], MemValue::U32(a as u32)).unwrap();
            }
            // 3.x should be saturated.  So adding more should trigger at least on hybrid merge.
            for a in 32..=64 {
                trie = trie.deep_insert([3, a], MemValue::U32(a as u32)).unwrap();
            }
            // Test post-commit insertions.
            for a in 0..=64 {
                assert_eq!(
                    Some(MemValue::U32(a as u32)),
                    trie.deep_query_value([3, a]).unwrap()
                );
            }
        }
    }

    #[tokio::test]
    async fn later_insertion_overwrites_earlier_insertion() {
        let space = MemSpace::new();
        let trie = SpaceTrie::connect(&space)
            .unwrap()
            .insert(-1, MemValue::U32(42))
            .unwrap()
            .insert(-1, MemValue::U32(43))
            .unwrap();
        let value = trie.query_value(-1).unwrap();
        assert_eq!(Some(MemValue::U32(43)), value);
    }

    #[tokio::test]
    async fn different_keys_have_different_values() {
        let space = MemSpace::new();
        let mut trie = SpaceTrie::connect(&space).unwrap();
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
        let space = MemSpace::new();
        let mut trie = SpaceTrie::connect(&space).unwrap();
        for e in 0..=33 {
            trie = trie.deep_insert([e, e], MemValue::U32(e as u32)).unwrap();
        }
        {
            let value = trie.deep_query_value([4]).unwrap();
            let Some(MemValue::MapBase(map_base)) = value else {
                panic!("expected map_base");
            };
            assert_eq!(1, map_base.map().slot_count());
        }
        {
            let value = trie.deep_query_value([4, 4]).unwrap();
            assert_eq!(Some(MemValue::U32(4)), value);
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
            let result = trie.deep_query_value([4, 4, -1]);
            let Err(_) = result else {
                panic!("deep_query_value should fail for invalid keys");
            };
        }
    }
}
