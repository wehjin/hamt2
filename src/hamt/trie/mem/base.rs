use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::mem::slot::MemSlot;
use crate::hamt::trie::mem::value::MemValue;
use crate::space;
use crate::QueryError;
use crate::TransactError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemBase {
    pub slots: Vec<MemSlot>,
}

impl MemBase {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_mb(map_base: TrieMapBase) -> Result<Self, TransactError> {
        let slot = MemSlot::MapBase(map_base);
        let slots = vec![slot];
        Ok(Self { slots })
    }
    pub fn new_kv(key: TrieKey, value: MemValue) -> Result<Self, TransactError> {
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
        value: MemValue,
    ) -> Result<Self, TransactError> {
        let slot = MemSlot::one_kv(key, value)?;
        let MemBase { mut slots } = self;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn replace_value(self, base_index: usize, value: MemValue) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value)?;
        slots.insert(base_index, slot);
        Ok(Self { slots })
    }
    pub fn query_key_values(
        &self,
        reader: &impl space::Read,
    ) -> Result<Vec<(i32, MemValue)>, QueryError> {
        let mut result = Vec::new();
        for slot in &self.slots {
            let key_values = slot.query_key_values(reader)?;
            result.extend(key_values);
        }
        Ok(result)
    }

    pub fn kick_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let slot = {
            let MemSlot::KeyValue(b_key, b_value) = pre_slot else {
                unreachable!("Should be a key-value slot, not a map-base slot:")
            };
            let b_key = key.sync(b_key);
            debug_assert!(b_key.i32() != key.i32());
            MemSlot::two_kv(b_key.next(), b_value, key.next(), value, reader)?
        };
        slots.insert(base_index, slot);
        let post = Self { slots };
        Ok(post)
    }

    pub fn merge_kv(
        self,
        base_index: usize,
        key: TrieKey,
        value: MemValue,
        reader: &impl space::Read,
    ) -> Result<Self, TransactError> {
        let MemBase { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let post_slot = {
            let MemSlot::MapBase(map_base) = pre_slot else {
                unreachable!("Should be a map-base slot, not a key-value slot:")
            };
            let post_map_base = map_base.insert_kv(key.next(), value, reader)?;
            MemSlot::MapBase(post_map_base)
        };
        slots.insert(base_index, post_slot);
        let post_base = Self { slots };
        Ok(post_base)
    }
    pub fn as_slot(&self, base_index: usize) -> Result<&MemSlot, QueryError> {
        self.slots
            .get(base_index)
            .ok_or(QueryError::BaseIndexOutOfBounds(base_index))
    }
}
