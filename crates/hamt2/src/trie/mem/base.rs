use crate::TransactError;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use serde::{Deserialize, Serialize};
use std::ops::Index;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BaseId(i32);
pub trait BaseStorage {
    // Read the next available base id.
    fn max_id(&self) -> BaseId;

    // Store a base
    fn append(&mut self, base: &Base) -> impl Future<Output = Result<BaseId, StorageError>>;
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Base {
    pub slots: Vec<MemSlot>,
}

impl Base {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: TrieKey, value: MemValue) -> Self {
        let slot = MemSlot::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }
}

impl Base {
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(self, base_index: usize, slot: MemSlot) -> Self {
        let mut slots = self.slots;
        slots.insert(base_index, slot);
        Self { slots }
    }
    pub fn replace_value(self, base_index: usize, value: MemValue) -> Self {
        let Base { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value);
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub fn kick_kv(self, base_index: usize, key: TrieKey, value: MemValue) -> Self {
        let Base { mut slots } = self;
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
    ) -> Result<Self, TransactError> {
        let Base { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let post_slot = {
            let MemSlot::MapBase(map_base) = pre_slot else {
                unreachable!("Should be a map-base slot, not a key-value slot:")
            };
            let post_map_base = map_base.insert_kv(key.next(), value).await?;
            MemSlot::MapBase(post_map_base)
        };
        slots.insert(base_index, post_slot);
        let post_base = Self { slots };
        Ok(post_base)
    }
}

impl Index<usize> for Base {
    type Output = MemSlot;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
