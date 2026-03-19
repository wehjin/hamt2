use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map_base::TrieMapBase;
use crate::hamt::trie::mem::slot::MemSlot;
use crate::hamt::trie::value::TrieValue;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemBase {
    pub slots: Vec<MemSlot>,
}

impl MemBase {
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn empty() -> Self {
        Self { slots: vec![] }
    }
    pub fn one_mb(map_base: TrieMapBase) -> Result<Self, TransactError> {
        let slot = MemSlot::MapBase(map_base);
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn insert_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let MemBase { mut slots } = self;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: TrieValue,
    ) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index);
        let slot = slot.merge_kv(key, value)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        self.slots
            .get(base_index)
            .ok_or(QueryError::BaseIndexOutOfBounds(base_index))
    }
}
