use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::mem::base::MemBase;
use crate::hamt::trie::mem::value::MemValue;
use crate::space;
use crate::QueryError;
use crate::TransactError;
use serde::{Deserialize, Serialize};
use crate::hamt::trie::core::query::QueryKeysValues;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemSlot {
    KeyValue(i32, MemValue),
    MapBase(TrieMapBase),
}

impl MemSlot {
    pub fn one_kv(key: TrieKey, value: MemValue) -> Result<Self, TransactError> {
        Ok(Self::KeyValue(key.i32(), value))
    }
    pub fn two_kv(
        a_key: TrieKey,
        a_value: MemValue,
        b_key: TrieKey,
        b_value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        debug_assert!(a_key.i32() != b_key.i32());
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index == b_map_index {
            let map = TrieMap::set_map_index_bit(a_map_index);
            let slot = MemSlot::two_kv(a_key.next(), a_value, b_key.next(), b_value, reader)?;
            let base = MemBase { slots: vec![slot] };
            Ok(MemSlot::MapBase(TrieMapBase::Mem(map, base)))
        } else {
            let map_base = TrieMapBase::two_kv(a_key, a_value, b_key, b_value)?;
            Ok(MemSlot::MapBase(map_base))
        }
    }
    pub fn replace_value(self, value: MemValue) -> Result<Self, TransactError> {
        let MemSlot::KeyValue(key, _value) = self else {
            return Err(TransactError::InvalidSlotType);
        };
        let slot = MemSlot::KeyValue(key, value);
        Ok(slot)
    }
    pub fn query_key_values(
        &self,
        reader: &impl space::Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        match self {
            MemSlot::KeyValue(key, value) => Ok(vec![(*key, value.clone())]),
            MemSlot::MapBase(map_base) => map_base.query_keys_values(reader),
        }
    }
    pub fn query_value<'a>(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<MemValue>, QueryError> {
        match self {
            MemSlot::KeyValue(k, v) => {
                if *k != key.i32() {
                    Ok(None)
                } else {
                    Ok(Some(v.clone()))
                }
            }
            MemSlot::MapBase(map_base) => {
                let value = map_base.query_value(key.next(), reader)?;
                Ok(value)
            }
        }
    }
    pub fn test_kv(&self, key: &TrieKey, value: &MemValue) -> KvTest {
        match self {
            MemSlot::KeyValue(slot_key, slot_value) => {
                if key.i32() == *slot_key {
                    if value == slot_value {
                        KvTest::SameValue
                    } else {
                        KvTest::ConflictOldValue
                    }
                } else {
                    KvTest::ConflictKeyValue
                }
            }
            MemSlot::MapBase(_) => KvTest::ConflictMapBase,
        }
    }
}

pub enum KvTest {
    SameValue,
    ConflictOldValue,
    ConflictKeyValue,
    ConflictMapBase,
}
