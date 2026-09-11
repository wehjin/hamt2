use crate::trie::core::map::TrieMap;
use crate::trie::mem::base::MemBase;
use serde::{Deserialize, Serialize};

pub mod cons;
pub mod insert;
pub mod query;
pub mod write;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TrieMapBase {
    pub map: TrieMap,
    pub base: MemBase,
}

#[cfg(test)]
mod tests {
    use crate::space::Space;
    use crate::space::mem::MemSpace;
    use crate::trie::core::key::TrieKey;
    use crate::trie::core::map::TrieMap;
    use crate::trie::core::map_base::*;
    use crate::trie::mem::base::MemBase;
    use crate::trie::mem::value::MemValue;
    use crate::trie::space::root::SpaceRoot;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_stream_kvs_empty_map() {
        let map_base = TrieMapBase {
            map: TrieMap::empty(),
            base: MemBase::new(),
        };
        let stream = map_base.kv_stream();
        let kvs = stream.collect::<Vec<_>>().await;
        assert!(kvs.is_empty());
    }
    #[tokio::test]
    async fn test_stream_kvs_one_slot() {
        let key = TrieKey::new(0);
        let value = MemValue::from(11);
        let map_base = {
            let map = TrieMap::set_key_bit(key);
            let base = MemBase::new_kv(key, value.clone());
            TrieMapBase { map, base }
        };
        let stream = map_base.kv_stream();
        let kvs = stream.collect::<Vec<_>>().await;
        assert_eq!(vec![(key.i32(), value)], kvs);
    }

    #[tokio::test]
    async fn test_stream_kvs_many_slots() -> anyhow::Result<()> {
        let test_kvs = (0..35)
            .map(|i| (i, MemValue::from(i as u32)))
            .collect::<Vec<_>>();
        let mut space = MemSpace::new();
        // Setup
        let map_base = {
            let mut map_base = {
                let key = TrieKey::new(test_kvs[0].0);
                let value = test_kvs[0].1.clone();
                let map = TrieMap::set_key_bit(key);
                let base = MemBase::new_kv(key, value.clone());
                TrieMapBase { map, base }
            };
            for kv in &test_kvs[1..] {
                let key = TrieKey::new(kv.0);
                let value = kv.1.clone();
                map_base = map_base.insert_kv(key, value).await?;
            }
            map_base
        };
        // Test pre-save kev values.
        {
            let stream = map_base.kv_stream();
            let mut kvs = stream.collect::<Vec<_>>().await;
            kvs.sort_by_key(|(k, _)| *k);
            assert_eq!(test_kvs, kvs);
        }
        // Write to space.
        let root_addr = {
            let mut extend = space.extend().await?;
            let space_root = SpaceRoot::from_trie_map_base(map_base, &mut extend)?;
            let root_addr = space_root.into_root_addr(&mut extend)?;
            extend.commit(&mut space).await?;
            root_addr.to_u32()
        };
        let reader = space.read().await?;
        let map_base = SpaceRoot::from_root_addr(root_addr, &reader)
            .await?
            .into_mem(&reader)
            .await?;
        {
            let stream = map_base.kv_stream();
            let mut kvs = stream.collect::<Vec<_>>().await;
            kvs.sort_by_key(|(k, _)| *k);
            assert_eq!(test_kvs, kvs);
        }
        Ok(())
    }
}
