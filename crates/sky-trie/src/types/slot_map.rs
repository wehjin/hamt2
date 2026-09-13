use crate::types::HashKey;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SlotMap(pub u32);

impl SlotMap {
    pub fn u32(&self) -> u32 {
        self.0
    }
    pub fn slot_count(&self) -> usize {
        self.0.count_ones() as usize
    }
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn set_key_bit(key: HashKey) -> Self {
        let map = key.to_map_bit();
        Self(map)
    }
    pub fn set_map_index_bit(map_index: u8) -> Self {
        let bit = HashKey::map_bit_from_map_index(map_index);
        Self(bit)
    }
    pub fn with_key(&self, key: HashKey) -> Self {
        let bit = key.to_map_bit();
        let map = self.0 | bit;
        Self(map)
    }

    pub fn is_present(&self, key: HashKey) -> bool {
        (key.to_map_bit() & self.0) != 0
    }

    pub fn count_left(&self, key: HashKey) -> usize {
        let map_index = key.map_index();
        let mask = !(0xFFFFFFFFu32 >> map_index);
        let left_count = u32::count_ones(mask & self.0) as usize;
        left_count
    }
    pub fn try_base_index(&self, key: HashKey) -> Option<usize> {
        if self.is_present(key) {
            let base_index = self.count_left(key);
            Some(base_index)
        } else {
            None
        }
    }
}

impl fmt::Debug for SlotMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TrieMap")
            .field(&format_args!("{:032b}", &self.0))
            .finish()
    }
}
