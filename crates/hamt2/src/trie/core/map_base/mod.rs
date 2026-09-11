use crate::trie::base::BaseId;
use crate::trie::core::map::TrieMap;
use serde::{Deserialize, Serialize};

pub mod cons;
pub mod insert;
pub mod query;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MapBase {
    pub map: TrieMap,
    pub base: BaseId,
}

#[cfg(test)]
mod tests {
    use crate::trie::base_storage::mem::MemBaseStorage;
    use crate::trie::core::key::TrieKey;
    use crate::trie::core::map_base::*;
    use crate::trie::mem::value::MemValue;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_stream_kvs_empty_map() {
        let storage = MemBaseStorage::new();
        let map_base = MapBase::empty();
        let stream = map_base.kv_stream(&storage);
        let kvs = stream.collect::<Vec<_>>().await;
        assert!(kvs.is_empty());
    }
    #[tokio::test]
    async fn test_stream_kvs_one_slot() {
        let key = TrieKey::new(0);
        let value = MemValue::from(11);
        let mut storage = MemBaseStorage::new();
        let map_base = MapBase::one_kv(key, value.clone(), &mut storage).await;
        let stream = map_base.kv_stream(&storage);
        let kvs = stream.collect::<Vec<_>>().await;
        assert_eq!(vec![(key.i32(), value)], kvs);
    }

    #[tokio::test]
    async fn test_stream_kvs_many_slots() -> anyhow::Result<()> {
        let test_kvs = (0..35)
            .map(|i| (i, MemValue::from(i as u32)))
            .collect::<Vec<_>>();
        let mut storage = MemBaseStorage::new();
        let map_base = {
            let mut map_base = {
                let key = TrieKey::new(test_kvs[0].0);
                let value = test_kvs[0].1.clone();
                MapBase::one_kv(key, value, &mut storage).await
            };
            for kv in &test_kvs[1..] {
                let key = TrieKey::new(kv.0);
                let value = kv.1.clone();
                map_base = map_base.insert_kv(key, value, &mut storage).await?;
            }
            map_base
        };
        let stream = map_base.kv_stream(&storage);
        let mut kvs = stream.collect::<Vec<_>>().await;
        kvs.sort_by_key(|(k, _)| *k);
        assert_eq!(test_kvs, kvs);
        Ok(())
    }
}
