use crate::space::Read;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::value::MemValue;
use crate::trie::space::map_base::SpaceMapBase;
use crate::{space, QueryError};

impl TrieMapBase {
    pub async fn query_value(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<MemValue>, QueryError> {
        let value = match self {
            TrieMapBase::Mem(map, base) => match map.try_base_index(key) {
                Some(base_index) => Box::pin(base[base_index].query_value(key, reader)).await?,
                None => None,
            },
            TrieMapBase::Space(slot_value) => {
                SpaceMapBase::assert(*slot_value)
                    .query_value(key, reader)
                    .await?
            }
        };
        Ok(value)
    }
}

impl QueryKeysValues for TrieMapBase {
    async fn query_keys_values(
        &self,
        reader: &impl Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut out = Vec::new();
        match self {
            TrieMapBase::Mem(map, base) => {
                let slot_count = map.slot_count();
                debug_assert_eq!(slot_count, base.len());
                for base_index in 0..slot_count {
                    let keys_values = Box::pin(base[base_index].query_key_values(reader)).await?;
                    out.extend(keys_values);
                }
            }
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                out.extend(map_base.query_keys_values(reader).await?);
            }
        }
        Ok(out)
    }
}
