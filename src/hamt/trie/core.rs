use crate::client::TransactError;
use crate::hamt::trie::key::TrieKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrieMap(pub u32);

impl TrieMap {
    pub fn new(key: TrieKey) -> Self {
        let map = key.to_map_bit();
        Self(map)
    }

    pub fn is_present(&self, key: TrieKey) -> bool {
        (key.to_map_bit() & self.0) != 0
    }

    pub fn to_base_index(&self, key: TrieKey) -> Option<usize> {
        if self.is_present(key) {
            let map_index = key.map_index();
            let mask = !(0xFFFFFFFFu32 >> map_index);
            let base_index = u32::count_ones(mask & self.0) as usize;
            Some(base_index)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrieValue(u32);

impl TrieValue {
    pub fn new(value: u32) -> Result<Self, TransactError> {
        if value & 0x80000000 != 0 {
            return Err(TransactError::HighBitInValue(value));
        }
        let value = value << 1;
        Ok(Self(value))
    }
    pub fn to_value(&self) -> u32 {
        self.0 >> 1
    }
}