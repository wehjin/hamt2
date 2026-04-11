use crate::space::core::reader::SlotValue;
use crate::space::TableAddr;
use crate::trie::core::map::TrieMap;
use crate::trie::space::key_value::SpaceKeyValue;
use crate::trie::space::map_base::SpaceMapBase;

#[derive(Debug, Clone, Copy)]
pub struct SpaceSlot(SlotValue);

impl SpaceSlot {
    pub fn assert(slot_value: SlotValue) -> Self {
        Self(slot_value)
    }

    pub fn into_slot_value(self) -> SlotValue {
        self.0
    }

    pub fn from_key_value(key: i32, value: u32) -> Self {
        let left = value;
        debug_assert!(key >= 0);
        let right = key as u32;
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }
    pub fn is_key_value(&self) -> bool {
        (self.0.right() & 0x80000000) == 0
    }
    pub fn to_key_and_value(&self) -> (i32, u32) {
        debug_assert!(self.is_key_value());
        let value = self.0.left();
        let key = (self.0.right() & 0x07fff_ffff) as i32;
        (key, value)
    }

    pub fn try_key_value(&self) -> Option<SpaceKeyValue> {
        if self.is_key_value() {
            let key_value = SpaceKeyValue::from(self.clone());
            Some(key_value)
        } else {
            None
        }
    }

    pub fn from_map_base(map: TrieMap, base_addr: TableAddr) -> Self {
        let left = map.u32();
        let right = {
            let base_addr = base_addr.to_u32();
            debug_assert_eq!(0, base_addr & 0x8000_0000);
            base_addr | 0x8000_0000
        };
        let slot_value = SlotValue::from((left, right));
        Self(slot_value)
    }

    pub fn is_map_base(&self) -> bool {
        (self.0.right() & 0x80000000) == 0x80000000
    }

    pub fn try_map_base(&self) -> Option<SpaceMapBase> {
        if self.is_map_base() {
            let map = self.0.left();
            let base = self.0.right() & 0x7fffffff;
            let map_base = SpaceMapBase::new(TrieMap(map), TableAddr(base));
            Some(map_base)
        } else {
            None
        }
    }
}
