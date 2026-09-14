pub(crate) mod crate_services;
pub mod error;
pub mod prelude;
pub mod storage_trie_query;
mod trie;
pub mod trie_reader;
pub mod trie_storage;
pub mod types;

pub use error::*;
pub use sky_types::trie::TrieQuery;
pub use storage_trie_query::StorageTrieQuery;
pub use trie::*;
pub use trie_reader::TrieReader;

#[cfg(test)]
mod tests {
    use crate::trie_storage::ReadTrieStorage;
    use crate::trie_storage::file::FileTrieStorage;
    use crate::trie_storage::mem::MemTrieStorage;
    use sky_types::trie::TrieValue
;
    use crate::{Trie, TrieQuery, TrieReader};

    #[tokio::test]
    async fn file_trie_works() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        {
            let storage = FileTrieStorage::new(dir.path())?;
            let trie = Trie::connect(storage).await?;
            trie.insert(1, TrieValue::U32(1)).await?.commit().await?;
        }
        {
            let storage = FileTrieStorage::load(dir.path())?;
            let trie = Trie::connect(storage).await?;
            let value = trie.query_value(1).await?;
            assert_eq!(Some(TrieValue::U32(1)), value);
        }
        Ok(())
    }

    #[tokio::test]
    async fn max_u32_works_as_value() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemTrieStorage::new()).await?;
        trie = trie.insert(1, TrieValue::U32(u32::MAX)).await?;
        assert_eq!(
            vec![(1, TrieValue::U32(u32::MAX))],
            trie.query_keys_values().await?
        );
        trie = trie.commit().await?;
        {
            let storage = trie.close();
            let trie = Trie::connect(storage).await?;
            assert_eq!(
                vec![(1, TrieValue::U32(u32::MAX))],
                trie.query_keys_values().await?
            );
        }
        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "assertion failed: value >= 0")]
    async fn negative_i32_does_not_work_as_key() {
        let trie = Trie::connect(MemTrieStorage::new()).await.expect("connect");
        let _trie = trie.insert(-1, TrieValue::U32(10)).await.expect("insert");
    }

    #[tokio::test]
    async fn query_key_values_works() {
        let mut trie = Trie::connect(MemTrieStorage::new()).await.expect("connect");
        trie = trie.insert(1, TrieValue::U32(1)).await.expect("insert");
        trie = trie.insert(2, TrieValue::U32(2)).await.expect("insert");
        let key_values = trie.query_keys_values().await.expect("all_keys_values");
        let mut key_values = key_values
            .into_iter()
            .map(|kv| {
                let TrieValue::U32(value) = kv.1 else {
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
        // Commit once.
        let storage = {
            let mut trie = Trie::connect(MemTrieStorage::new()).await.unwrap();
            trie = trie.insert(1, TrieValue::U32(42)).await.unwrap();
            trie = trie
                .deep_insert([2, 42], TrieValue::U32(242), false)
                .await
                .unwrap();
            trie = trie.commit().await.unwrap();
            trie.close()
        };
        // Commit again.
        let storage = {
            let mut trie = Trie::connect(storage).await.unwrap();
            trie = trie.insert(1, TrieValue::U32(84)).await.unwrap();
            trie = trie.commit().await.expect("commit");
            trie.close()
        };
        // Query from both commits.
        {
            let trie = Trie::connect(storage).await.unwrap();
            assert_eq!(Some(TrieValue::U32(84)), trie.query_value(1).await.unwrap());
            assert_eq!(
                Some(TrieValue::U32(242)),
                trie.deep_query_value([2, 42]).await.unwrap()
            );
        }
    }

    #[tokio::test]
    async fn read_trie_queries_work() -> anyhow::Result<()> {
        let storage = {
            let mut trie = Trie::connect(MemTrieStorage::new()).await?;
            trie = trie.insert(1, TrieValue::U32(42)).await?;
            trie = trie
                .deep_insert([2, 42], TrieValue::U32(242), false)
                .await?;
            trie.commit().await?.close()
        };
        let view_storage = storage.snapshot();
        let read_trie = TrieReader::connect(view_storage).await?;
        assert_eq!(Some(TrieValue::U32(42)), read_trie.query_value(1).await?);
        assert_eq!(
            Some(TrieValue::U32(242)),
            read_trie.deep_query_value([2, 42]).await?
        );
        Ok(())
    }

    #[tokio::test]
    async fn persistence_works() {
        // Commit some values.
        let storage = {
            let mut trie = Trie::connect(MemTrieStorage::new()).await.unwrap();
            trie = trie.insert(100, TrieValue::U32(42)).await.unwrap();
            for a in 0..=32 {
                trie = trie
                    .deep_insert([3, a], TrieValue::U32(a as u32), false)
                    .await
                    .unwrap();
            }
            trie = trie.commit().await.unwrap();
            trie.close()
        };
        // Test commited values.
        let storage = {
            let trie = Trie::connect(storage).await.unwrap();
            assert_eq!(
                Some(TrieValue::U32(42)),
                trie.query_value(100).await.unwrap()
            );
            for a in 0..=32 {
                assert_eq!(
                    Some(TrieValue::U32(a as u32)),
                    trie.deep_query_value([3, a]).await.unwrap()
                );
            }
            trie.close()
        };
        // Deep insert values to saturate root blocks in deep tries.
        {
            let mut trie = Trie::connect(storage).await.unwrap();
            // Use at least 33 keys so that the root blook in the first trie is saturated.
            for i in 0..35 {
                let e = 5 + i;
                trie = trie
                    .deep_insert([e, 0], TrieValue::U32(e as u32), false)
                    .await
                    .unwrap();
            }
            // Use at least 33 keys so that the root block in the second trie is saturated.
            for i in 0..35 {
                let a = 3 + i;
                trie = trie
                    .deep_insert([4, a], TrieValue::U32(a as u32), false)
                    .await
                    .unwrap();
            }
            // 3.x should be saturated.  So adding more should trigger at least on hybrid merge.
            for a in 32..=64 {
                trie = trie
                    .deep_insert([3, a], TrieValue::U32(a as u32), false)
                    .await
                    .unwrap();
            }
            // Test post-commit insertions.
            for a in 0..=64 {
                assert_eq!(
                    Some(TrieValue::U32(a as u32)),
                    trie.deep_query_value([3, a]).await.unwrap()
                );
            }
        }
    }

    #[tokio::test]
    async fn later_insertion_overwrites_earlier_insertion() {
        let trie = Trie::connect(MemTrieStorage::new())
            .await
            .unwrap()
            .insert(1, TrieValue::U32(42))
            .await
            .unwrap()
            .insert(1, TrieValue::U32(43))
            .await
            .unwrap();
        let value = trie.query_value(1).await.unwrap();
        assert_eq!(Some(TrieValue::U32(43)), value);
    }

    #[tokio::test]
    async fn different_keys_have_different_values() {
        let mut trie = Trie::connect(MemTrieStorage::new()).await.unwrap();
        // 33 keys will saturate the root block.
        let keys = (0..=32).collect::<Vec<_>>();
        for i in &keys {
            trie = trie.insert(*i, TrieValue::U32(*i as u32)).await.unwrap();
        }
        let mut values = Vec::new();
        for i in &keys {
            let value = trie
                .query_value(*i)
                .await
                .expect(&format!("query for key: {}", i));
            values.push(value);
        }
        let expected = keys
            .iter()
            .map(|i| Some(TrieValue::U32(*i as u32)))
            .collect::<Vec<_>>();
        assert_eq!(expected, values);
    }

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut trie = Trie::connect(MemTrieStorage::new()).await.unwrap();
        for e in 0..=33 {
            trie = trie
                .deep_insert([e, e], TrieValue::U32(e as u32), false)
                .await
                .unwrap();
        }
        {
            let value = trie.deep_query_value([4]).await.unwrap();
            let Some(TrieValue::SubTrie(map_base)) = value else {
                panic!("expected map_base");
            };
            assert_eq!(1, map_base.map.slot_count());
        }
        {
            let value = trie.deep_query_value([4, 4]).await.unwrap();
            assert_eq!(Some(TrieValue::U32(4)), value);
        }
        {
            let value = trie.deep_query_value([4, 1]).await.unwrap();
            assert_eq!(None, value);
        }
        {
            let value = trie.deep_query_value([5, 2]).await.unwrap();
            assert_eq!(None, value);
        }
    }

    #[tokio::test]
    #[should_panic(expected = "assertion failed: value >= 0")]
    async fn deep_query_fails_for_invalid_key() {
        let trie = Trie::connect(MemTrieStorage::new()).await.unwrap();
        let _result = trie.deep_query_value([4, 4, -1]).await;
    }
}
