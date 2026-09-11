use crate::trie::base::Base;
use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadWrite};
use crate::trie::core::key::TrieKey;
use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::value::MemValue;
use crate::QueryError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemSlot {
    KeyValue(i32, MemValue),
    MapBase(TrieMapBase),
}

impl MemSlot {
    pub fn one_kv(key: TrieKey, value: MemValue) -> Self {
        Self::KeyValue(key.i32(), value)
    }
    pub async fn two_kv(
        a_key: TrieKey,
        a_value: MemValue,
        b_key: TrieKey,
        b_value: MemValue,
        storage: &mut impl BaseStorageReadWrite,
    ) -> Self {
        debug_assert!(a_key.i32() != b_key.i32());
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index == b_map_index {
            let map = TrieMap::set_map_index_bit(a_map_index);
            let slot = Box::pin(MemSlot::two_kv(
                a_key.next(),
                a_value,
                b_key.next(),
                b_value,
                storage,
            ))
            .await;
            let base = Base { slots: vec![slot] };
            let id = storage.append(&base).await.expect("append base");
            MemSlot::MapBase(TrieMapBase { map, base: id })
        } else {
            let map_base = TrieMapBase::two_kv(a_key, a_value, b_key, b_value, storage).await;
            MemSlot::MapBase(map_base)
        }
    }
    pub fn replace_value(self, value: MemValue) -> Self {
        let MemSlot::KeyValue(key, _value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        MemSlot::KeyValue(key, value)
    }
    pub async fn query_key_values(
        &self,
        storage: &impl BaseStorageRead,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        match self {
            MemSlot::KeyValue(key, value) => Ok(vec![(*key, value.clone())]),
            MemSlot::MapBase(map_base) => map_base.query_keys_values(storage).await,
        }
    }
    pub async fn query_value(
        &self,
        key: TrieKey,
        storage: &impl BaseStorageRead,
    ) -> Result<Option<MemValue>, QueryError> {
        match self {
            MemSlot::KeyValue(k, v) => {
                if *k != key.i32() {
                    Ok(None)
                } else {
                    Ok(Some(v.clone()))
                }
            }
            MemSlot::MapBase(map_base) => map_base.query_value(key.next(), storage).await,
        }
    }
    pub fn test_kv(&self, key: &TrieKey, value: &MemValue) -> KvTest {
        match self {
            MemSlot::KeyValue(slot_key, slot_value) => {
                if key.i32() == *slot_key {
                    if value == slot_value {
                        KvTest::SameValue
                    } else {
                        KvTest::ValueConflict
                    }
                } else {
                    KvTest::KeyConflict
                }
            }
            MemSlot::MapBase(_) => KvTest::MapBaseConflict,
        }
    }
}

pub enum KvTest {
    SameValue,
    ValueConflict,
    KeyConflict,
    MapBaseConflict,
}
