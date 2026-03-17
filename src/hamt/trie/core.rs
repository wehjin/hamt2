use crate::client::TransactError;
use crate::hamt::trie::key::TrieKey;
use std::fmt;
use std::fmt::Formatter;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TrieMap(pub u32);

impl fmt::Debug for TrieMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TrieMap")
            .field(&format_args!("{:032b}", &self.0))
            .finish()
    }
}

impl TrieMap {
    pub fn set_key_bit(key: TrieKey) -> Self {
        let map = key.to_map_bit();
        Self(map)
    }
    pub fn set_map_index_bit(map_index: u8) -> Self {
        let bit = TrieKey::map_bit_from_map_index(map_index);
        Self(bit)
    }
    pub fn with_key(&self, key: TrieKey) -> Self {
        let bit = key.to_map_bit();
        let map = self.0 | bit;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TrieValue(u32);

impl fmt::Debug for TrieValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TrieValue")
            .field(&format_args!("{}", self.0 >> 1))
            .finish()
    }
}

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
