use crate::space;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::value::MemValue;
use crate::QueryError;
use crate::TransactError;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemSlot {
    KeyValue(TrieKey, TrieValue),
    MapBase(TrieMapBase),
}

impl MemSlot {
    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Ok(Self::KeyValue(key, value))
    }
    pub fn two_kv(
        a_key: TrieKey,
        a_value: TrieValue,
        b_key: TrieKey,
        b_value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let (a_map_index, b_map_index) = (a_key.map_index(), b_key.map_index());
        if a_map_index != b_map_index {
            let map_base = TrieMapBase::two_kv(a_key, a_value, b_key, b_value, reader)?;
            Ok(MemSlot::MapBase(map_base))
        } else {
            let map_index = a_map_index;
            let inner_slot = MemSlot::two_kv(a_key.next(), a_value, b_key.next(), b_value, reader)?;
            let map_base = TrieMapBase::one_slot(map_index, inner_slot)?;
            Ok(MemSlot::MapBase(map_base))
        }
    }
    pub fn replace_value(self, value: TrieValue) -> Result<Self, TransactError> {
        let MemSlot::KeyValue(key, _value) = self else {
            return Err(TransactError::InvalidSlotType);
        };
        let slot = MemSlot::KeyValue(key, value);
        Ok(slot)
    }
    pub fn merge_kv(
        self,
        key: TrieKey,
        value: TrieValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        match self {
            MemSlot::KeyValue(b_key, b_value) => {
                let slot = MemSlot::two_kv(b_key.next(), b_value, key.next(), value, reader)?;
                Ok(slot)
            }
            MemSlot::MapBase(map_base) => {
                let map_base = map_base.insert_kv(key.next(), value, reader)?;
                let slot = MemSlot::MapBase(map_base);
                Ok(slot)
            }
        }
    }
    pub fn query_key_values(
        &self,
        reader: &impl space::Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        match self {
            MemSlot::KeyValue(key, value) => Ok(vec![(key.i32(), value.to_mem_value(reader)?)]),
            MemSlot::MapBase(map_base) => map_base.query_key_values(reader),
        }
    }
    pub fn query_value<'a>(
        &self,
        key: TrieKey,
        reader: &impl space::Read,
    ) -> Result<Option<TrieValue>, QueryError> {
        match self {
            MemSlot::KeyValue(k, v) => {
                if k.i32() != key.i32() {
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
    pub fn test_kv(&self, key: &TrieKey, value: &TrieValue) -> KvTest {
        match self {
            MemSlot::KeyValue(slot_key, slot_value) => {
                if key.same_i32(slot_key) {
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
