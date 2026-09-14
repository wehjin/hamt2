pub mod cons;
pub mod insert;
pub mod query;

pub use cons::*;
pub use insert::*;
pub use query::*;

#[cfg(test)]
mod tests {
    use crate::crate_services::map_base;
    use crate::crate_services::map_base::*;
    use crate::trie_storage::mem::MemTrieStorage;
    use crate::types::HashKey;
    use sky_types::trie::TrieValue
;
    use sky_types::trie::MapBase
;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_stream_kvs_empty_map() {
        let storage = MemTrieStorage::new();
        let map_base = MapBase::empty();
        let stream = kv_stream(map_base, storage);
        let kvs = stream.collect::<Vec<_>>().await;
        assert!(kvs.is_empty());
    }
    #[tokio::test]
    async fn test_stream_kvs_one_slot() {
        let key = HashKey::new(0);
        let value = TrieValue::from(11);
        let mut storage = MemTrieStorage::new();
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
        let mut storage = MemTrieStorage::new();
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
