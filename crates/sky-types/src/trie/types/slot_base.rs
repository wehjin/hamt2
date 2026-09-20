use crate::trie::{HashKey, Slot, TrieConfig, TrieValue};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub struct SlotBase<C: TrieConfig> {
    pub slots: Vec<Slot<C>>,
}

impl<C: TrieConfig> AsRef<SlotBase<C>> for SlotBase<C> {
    fn as_ref(&self) -> &SlotBase<C> {
        &self
    }
}

impl<C: TrieConfig> Deref for SlotBase<C> {
    type Target = Vec<Slot<C>>;

    fn deref(&self) -> &Self::Target {
        &self.slots
    }
}
impl<C: TrieConfig> DerefMut for SlotBase<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.slots
    }
}

impl<C: TrieConfig> SlotBase<C> {
    pub fn new() -> Self {
        Self { slots: vec![] }
    }
    pub fn new_kv(key: HashKey, value: TrieValue<C>) -> Self {
        let slot = Slot::<C>::one_kv(key, value);
        let slots = vec![slot];
        Self { slots }
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }
    pub fn insert_slot(&self, base_index: usize, slot: Slot<C>) -> Self {
        let mut slots = self.slots.clone();
        slots.insert(base_index, slot);
        Self { slots }
    }

    pub fn replace_slot(&self, base_index: usize, replacement: Slot<C>) -> Self {
        let mut new_slots = self.slots.clone();
        new_slots[base_index] = replacement;
        Self { slots: new_slots }
    }
}

impl<C: TrieConfig> Index<usize> for SlotBase<C> {
    type Output = Slot<C>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
