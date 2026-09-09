use crate::space;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::TransactError;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemBase {
    pub slots: Vec<MemSlot>,
}

impl MemBase {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: TrieKey, value: MemValue) -> Self {
        let slot = MemSlot::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }
}

impl MemBase {
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(self, base_index: usize, slot: MemSlot) -> Self {
        let mut slots = self.slots;
        slots.insert(base_index, slot);
        Self { slots }
    }
    pub fn replace_value(self, base_index: usize, value: MemValue) -> Self {
        let MemBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value);
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub fn kick_kv(self, base_index: usize, key: TrieKey, value: MemValue) -> Self {
        let MemBase { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let slot = {
            let MemSlot::KeyValue(b_key, b_value) = pre_slot else {
                unreachable!("Should be a key-value slot, not a map-base slot:")
            };
            let b_key = key.sync(b_key);
            debug_assert!(b_key.i32() != key.i32());
            MemSlot::two_kv(b_key.next(), b_value, key.next(), value)
        };
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub async fn merge_kv(
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
            let post_map_base = Box::pin(map_base.insert_kv(key.next(), value, reader)).await?;
            MemSlot::MapBase(post_map_base)
        };
        slots.insert(base_index, post_slot);
        let post_base = Self { slots };
        Ok(post_base)
    }
}

impl Index<usize> for MemBase {
    type Output = MemSlot;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
