use crate::hamt::space;
use crate::hamt::space::TableAddr;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::slot::MemSlot;
use crate::QueryError;
use crate::TransactError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemBase {
    pub slots: Vec<MemSlot>,
}

impl MemBase {
    pub fn load(
        addr: &TableAddr,
        count: usize,
        reader: &impl space::Read,
    ) -> Result<Self, space::ReadError> {
        let mut slots = Vec::with_capacity(count);
        for i in 0..count {
            let slot = reader.read_slot(addr, i)?;
            slots.push(slot.clone());
        }
        Ok(Self { slots })
    }
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_mb(map_base: TrieMapBase) -> Result<Self, TransactError> {
        let slot = MemSlot::MapBase(map_base);
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn new_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let slots = vec![slot];
        Ok(Self { slots })
    }
}

impl MemBase {
    pub fn len(&self) -> usize {
        self.slots.len()
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
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index);
        let slot = slot.merge_kv(key, value, reader)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        self.slots
            .get(base_index)
            .ok_or(QueryError::BaseIndexOutOfBounds(base_index))
    }
}
