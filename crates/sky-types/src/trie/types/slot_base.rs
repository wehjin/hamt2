use crate::trie::{HashKey, Slot, TrieValue};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotBase {
    pub slots: Vec<Slot>,
}

impl AsRef<SlotBase> for SlotBase {
    fn as_ref(&self) -> &SlotBase {
        &self
    }
}

impl Deref for SlotBase {
    type Target = Vec<Slot>;

    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}
impl DerefMut for SlotBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}

impl SlotBase {
    pub fn empty() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: HashKey, value: TrieValue) -> Self {
        let slot = Slot::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(&self, base_index: usize, slot: Slot) -> Self {
        let mut slots = self.slots.clone();
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub fn replace_slot(&self, base_index: usize, replacement: Slot) -> Self {
        let mut new_slots = self.slots.clone();
        new_slots[base_index] = replacement;
        Self { slots: new_slots }
    }
}

impl Index<usize> for SlotBase {
    type Output = Slot;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
