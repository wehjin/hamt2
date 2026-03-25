use crate::hamt::trie::core::map::TrieMap;
use crate::hamt::trie::space::map_base::{SpaceKeyValue, SpaceMapBase};
use crate::space::reader::SlotValue;
use crate::space::TableAddr;

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
    pub fn to_key_value(&self) -> Option<SpaceKeyValue> {
        if self.is_key_value() {
            let key = self.0.left() as i32;
            let value = (self.0.right() & 0x7fffffff);
            let key_value = SpaceKeyValue::new(key, value);
            Some(key_value)
        } else {
            None
        }
    }
    pub fn from_key_value(key: i32, value: u32) -> Self {
        let left = key as u32;
        let right = {
            debug_assert_eq!(0, value & 0x8000_0000);
            value & 0x7fffffff
        };
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }

    pub fn is_map_base(&self) -> bool {
        (self.0.right() & 0x80000000) == 0x80000000
    }
    pub fn to_map_base(&self) -> Option<SpaceMapBase> {
        if self.is_map_base() {
            let map = TrieMap(self.0.left());
            let base_addr = TableAddr(self.0.right() & 0x7fffffff);
            let map_base = SpaceMapBase::new(map, base_addr);
            Some(map_base)
        } else {
            None
        }
    }
    pub fn from_map_base(map: TrieMap, base_addr: TableAddr) -> Self {
        let left = map.u32();
        let right = {
            let pre_right = base_addr.u32();
            debug_assert_eq!(0, pre_right & 0x8000_0000);
            let right = pre_right | 0x8000_0000;
            right
        };
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }
}
