use crate::client::{QueryError, TransactError};
use crate::hamt::trie::core::TrieMap;
use crate::hamt::trie::core::TrieValue;
use crate::hamt::trie::key::TrieKey;

pub struct MemMapBase {
    pub map: TrieMap,
    pub base: MemBase,
}

impl MemMapBase {
    pub fn one_kv(key: TrieKey, value: u32) -> Result<Self, TransactError> {
        let map = TrieMap::new(key);
        let base = MemBase::one_kv(key, value)?;
        Ok(Self { map, base })
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        let base_index = self.map.to_base_index(key);
        if let Some(base_index) = base_index {
            let slot = self.base.as_slot(base_index)?;
            slot.query_value(key)
        } else {
            Ok(None)
        }
    }
}

pub struct MemBase {
    pub slots: Vec<MemSlot>,
}
impl MemBase {
    pub fn one_kv(key: TrieKey, value: u32) -> Result<Self, TransactError> {
        let slot = MemSlot::key_value(key, value)?;
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        self.slots
            .get(base_index)
            .ok_or(QueryError::BaseIndexOutOfBounds(base_index))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MemSlot {
    KeyValue(TrieKey, TrieValue),
    MapBase,
}

impl MemSlot {
    pub fn key_value(key: TrieKey, value: u32) -> Result<Self, TransactError> {
        let value = TrieValue::new(value)?;
        Ok(Self::KeyValue(key, value))
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        let MemSlot::KeyValue(k, v) = self else {
            return Err(QueryError::InvalidSlotType);
        };
        if k.i32() != key.i32() {
            Ok(None)
        } else {
            let value = v.to_value();
            Ok(Some(value))
        }
    }
}
