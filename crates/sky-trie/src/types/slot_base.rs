use crate::TrieInsertError;
use crate::crate_services::map_base;
use crate::trie_storage::ReadWriteTrieStorage;
use crate::types::HashKey;
use crate::types::slot::Slot;
use sky_types::trie::TrieValue
;
use serde::{Deserialize, Serialize};
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotBase {
    pub slots: Vec<Slot>,
}

impl SlotBase {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: HashKey, value: TrieValue) -> Self {
        let slot = Slot::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }
}

impl SlotBase {
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(self, base_index: usize, slot: Slot) -> Self {
        let mut slots = self.slots;
        slots.insert(base_index, slot);
        Self { slots }
    }
    pub fn replace_value(self, base_index: usize, value: TrieValue) -> Self {
        let SlotBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value);
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub async fn kick_kv(
        self,
        base_index: usize,
        key: HashKey,
        value: TrieValue,
        storage: &mut impl ReadWriteTrieStorage,
    ) -> Self {
        let SlotBase { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let slot = {
            let Slot::KeyValue(b_key, b_value) = pre_slot else {
                unreachable!("Should be a key-value slot, not a map-base slot:")
            };
            let b_key = key.sync(b_key);
            debug_assert!(b_key.i32() != key.i32());
            Slot::two_kv(b_key.next(), b_value, key.next(), value, storage).await
        };
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub async fn merge_kv(
        self,
        base_index: usize,
        key: HashKey,
        value: TrieValue,
        storage: &mut impl ReadWriteTrieStorage,
    ) -> Result<Self, TrieInsertError> {
        let SlotBase { mut slots } = self;
        let pre_slot = slots.remove(base_index);
        let post_slot = {
            let Slot::MapBase(pre_map_base) = pre_slot else {
                unreachable!("Should be a map-base slot, not a key-value slot:")
            };
            let post_map_base =
                map_base::insert_kv(pre_map_base, key.next(), value, storage).await?;
            Slot::MapBase(post_map_base)
        };
        slots.insert(base_index, post_slot);
        let post_base = Self { slots };
        Ok(post_base)
    }
}

impl Index<usize> for SlotBase {
    type Output = Slot;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
