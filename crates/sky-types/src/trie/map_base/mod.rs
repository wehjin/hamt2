pub mod cons;
pub mod insert;
pub mod query;

pub use cons::*;
pub use insert::*;
pub use query::*;

#[cfg(test)]
mod tests {
    use crate::storage::MemStorage;
    use crate::trie::map_base::{kv_stream, one_kv};
    use crate::trie::{HashKey, MapBase, TrieValue, map_base};
    use futures::StreamExt;

    #[tokio::test]
    async fn test_stream_kvs_empty_map() {
        let storage = MemStorage::new();
        let map_base = MapBase::empty();
        let stream = kv_stream(map_base, storage);
        let kvs = stream.collect::<Vec<_>>().await;
        assert!(kvs.is_empty());
    }
    #[tokio::test]
    async fn test_stream_kvs_one_slot() {
        let key = HashKey::new(0);
        let value = TrieValue::from(11);
        let mut storage = MemStorage::new();
        let map_base = one_kv(key, value.clone(), &mut storage).await;
        let stream = kv_stream(map_base, storage);
        let kvs = stream.collect::<Vec<_>>().await;
        assert_eq!(vec![(key.i32(), value)], kvs);
    }

    #[tokio::test]
    async fn test_stream_kvs_many_slots() -> anyhow::Result<()> {
        let test_kvs = (0..35)
            .map(|i| (i, TrieValue::from(i as u32)))
            .collect::<Vec<_>>();
        let mut storage = MemStorage::new();
        let map_base = {
            let mut map_base = {
                let key = HashKey::new(test_kvs[0].0);
                let value = test_kvs[0].1.clone();
                one_kv(key, value, &mut storage).await
            };
            for kv in &test_kvs[1..] {
                let key = HashKey::new(kv.0);
                let value = kv.1.clone();
                map_base = map_base::insert_kv(map_base, key, value, &mut storage).await?;
            }
            map_base
        };
        let stream = kv_stream(map_base, storage);
        let mut kvs = stream.collect::<Vec<_>>().await;
        kvs.sort_by_key(|(k, _)| *k);
        assert_eq!(test_kvs, kvs);
        Ok(())
    }
}
