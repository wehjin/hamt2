use crate::QueryError;
use crate::trie::trie_storage::{ReadTrieStorage, ReadWriteTrieStorage};
use crate::trie::types::slot_base::SlotBase;
use crate::trie::types::hash_key::HashKey;
use crate::trie::types::map_base::MapBase;
use crate::trie::types::slot_map::SlotMap;
use crate::trie::types::trie_value::TrieValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Slot {
    KeyValue(i32, TrieValue),
    MapBase(MapBase),
}

impl Slot {
    pub fn one_kv(key: HashKey, value: TrieValue) -> Self {
        Self::KeyValue(key.i32(), value)
    }
    pub async fn two_kv(
        a_key: HashKey,
        a_value: TrieValue,
        b_key: HashKey,
        b_value: TrieValue,
        storage: &mut impl ReadWriteTrieStorage,
    ) -> Self {
        debug_assert!(a_key.i32() != b_key.i32());
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index == b_map_index {
            let map = SlotMap::set_map_index_bit(a_map_index);
            let slot = Box::pin(Slot::two_kv(
                a_key.next(),
                a_value,
                b_key.next(),
                b_value,
                storage,
            ))
            .await;
            let base = SlotBase { slots: vec![slot] };
            let id = storage.append(&base).await.expect("append base");
            Slot::MapBase(MapBase { map, base: id })
        } else {
            let map_base = MapBase::two_kv(a_key, a_value, b_key, b_value, storage).await;
            Slot::MapBase(map_base)
        }
    }
    pub fn replace_value(self, value: TrieValue) -> Self {
        let Slot::KeyValue(key, _value) = self else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        Slot::KeyValue(key, value)
    }
    pub async fn query_key_values(
        &self,
        storage: &impl ReadTrieStorage,
    ) -> Result<Vec<(i32, TrieValue)>, QueryError> {
        match self {
            Slot::KeyValue(key, value) => Ok(vec![(*key, value.clone())]),
            Slot::MapBase(map_base) => map_base.query_keys_values(storage).await,
        }
    }
    pub async fn query_value(
        &self,
        key: HashKey,
        storage: &impl ReadTrieStorage,
    ) -> Result<Option<TrieValue>, QueryError> {
        match self {
            Slot::KeyValue(k, v) => {
                if *k != key.i32() {
                    Ok(None)
                } else {
                    Ok(Some(v.clone()))
                }
            }
            Slot::MapBase(map_base) => map_base.query_value(key.next(), storage).await,
        }
    }
    pub fn test_kv(&self, key: &HashKey, value: &TrieValue) -> KvTest {
        match self {
            Slot::KeyValue(slot_key, slot_value) => {
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
            Slot::MapBase(_) => KvTest::MapBaseConflict,
        }
    }
}

pub enum KvTest {
    SameValue,
    ValueConflict,
    KeyConflict,
    MapBaseConflict,
}
