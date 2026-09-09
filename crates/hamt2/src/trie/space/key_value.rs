use crate::space::core::reader::SlotValue;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;
use crate::trie::space::slots::SpaceSlot;

pub struct SpaceKeyValue(SpaceSlot);

impl SpaceKeyValue {
    pub fn new(key: i32, value: u32) -> Self {
        let space_slot = SpaceSlot::from_key_value(key, value);
        Self(space_slot)
    }
    pub fn into_slot_value(self) -> SlotValue {
        self.0.into_slot_value()
    }

    pub fn to_key_and_value(&self) -> (i32, u32) {
        self.0.to_key_and_value()
    }

    pub fn to_mem_slot(self) -> MemSlot {
        let (key, value) = self.to_key_and_value();
        let slot = MemSlot::KeyValue(key, MemValue::U32(value));
        slot
    }
    pub fn query_value(&self, key: TrieKey) -> Option<MemValue> {
        let (space_key, space_value) = self.to_key_and_value();
        if key.i32() == space_key {
            let value = space_value;
            Some(MemValue::U32(value))
        } else {
            None
        }
    }
}

impl From<SpaceSlot> for SpaceKeyValue {
    fn from(slot: SpaceSlot) -> Self {
        Self(slot)
    }
}
