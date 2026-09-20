use crate::trie::{HashKey, Slot, TrieValue};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut, Index};

pub trait TrieBase: AsRef<Self> {
    type HandleType;
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SlotBase<HandleType> {
    pub slots: Vec<Slot<HandleType>>,
}

impl<HandleType> TrieBase for SlotBase<HandleType> {
    type HandleType = HandleType;
}

impl<HandleType> AsRef<SlotBase<HandleType>> for SlotBase<HandleType> {
    fn as_ref(&self) -> &SlotBase<HandleType> {
        &self
    }
}

impl<HandleType> Deref for SlotBase<HandleType> {
    type Target = Vec<Slot<HandleType>>;

    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}
impl<HandleType> DerefMut for SlotBase<HandleType> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}

impl<HandleType: Clone + Eq + PartialEq> SlotBase<HandleType> {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: HashKey, value: TrieValue<HandleType>) -> Self {
        let slot = Slot::<HandleType>::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(&self, base_index: usize, slot: Slot<HandleType>) -> Self {
        let mut slots = self.slots.clone();
        slots.insert(base_index, slot);
        Self { slots }
    }

    /// Replaces the value of the slot at `base_index` with `value`.
    pub fn replace_value(self, base_index: usize, value: TrieValue<HandleType>) -> Self {
        let SlotBase { mut slots } = self;
        let slot = slots.remove(base_index).replace_value(value);
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub fn replace_slot(&self, base_index: usize, replacement: Slot<HandleType>) -> Self {
        let mut new_slots = vec![];
        for (pos, slot) in self.iter().enumerate() {
            if pos == base_index {
                new_slots.push(replacement.clone());
            } else {
                new_slots.push(slot.clone());
            }
        }
        Self { slots: new_slots }
    }
}

impl<HandleType> Index<usize> for SlotBase<HandleType> {
    type Output = Slot<HandleType>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
