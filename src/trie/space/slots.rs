use crate::space::core::reader::SlotValue;
use crate::space::TableAddr;
use crate::trie::core::map::TrieMap;
use crate::trie::space::map_base::{SpaceKeyValue, SpaceMapBase};

#[derive(Debug)]
pub struct SpaceSlot(SlotValue);

impl SpaceSlot {
    pub fn assert(slot_value: SlotValue) -> Self {
        Self(slot_value)
    }

    pub fn into_slot_value(self) -> SlotValue {
        self.0
    }

    pub fn is_key_value(&self) -> bool {
        (self.0.right() & 0x80000000) == 0
    }
    pub fn is_map_base(&self) -> bool {
        (self.0.right() & 0x80000000) == 0x80000000
    }

    pub fn to_key_value(&self) -> Option<SpaceKeyValue> {
        if self.is_key_value() {
            let key = self.0.left() as i32;
            let value = self.0.right() & 0x7fffffff;
            let key_value = SpaceKeyValue::new(key, value);
            Some(key_value)
        } else {
            None
        }
    }
    pub fn to_map_base(&self) -> Option<SpaceMapBase> {
        if self.is_map_base() {
            let map = self.0.left();
            let base = self.0.right() & 0x7fffffff;
            let map_base = SpaceMapBase::new(TrieMap(map), TableAddr(base));
            Some(map_base)
        } else {
            None
        }
    }

    pub fn from_key_value(key: i32, value: u32) -> Self {
        let left = key as u32;
        let right = {
            debug_assert_eq!(0, value & 0x8000_0000);
            value & 0x7fff_ffff
        };
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }
    pub fn from_map_base(map: TrieMap, base_addr: TableAddr) -> Self {
        let left = map.u32();
        let right = {
            let base_addr = base_addr.u32();
            debug_assert_eq!(0, base_addr & 0x8000_0000);
            base_addr | 0x8000_0000
        };
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }
}
