use crate::hamt::trie::core::map::TrieMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct TrieKey {
    value: i32,
    level: u32,
    hash: u32,
    hash_index: u32,
    map_index: u8,
}

impl TrieKey {
    pub const INVALID: Self = Self {
        value: -1,
        level: 0,
        hash: 0,
        hash_index: 0,
        map_index: 0,
    };

    pub fn new(value: i32) -> Self {
        let level = 1;
        let hash = hash_key(value as u32, level);
        let hash_index = 0;
        let map_index = map_index(hash, hash_index);
        Self {
            value,
            level,
            hash,
            hash_index,
            map_index,
        }
    }
    pub fn next(mut self) -> Self {
        self.hash_index += 1;
        if self.hash_index < 5 {
            self.map_index = map_index(self.hash, self.hash_index);
            self
        } else {
            self.level += 1;
            self.hash = hash_key(self.value as u32, self.level);
            self.hash_index = 0;
            self.map_index = map_index(self.hash, self.hash_index);
            self
        }
    }

    pub fn i32(&self) -> i32 {
        self.value
    }
    pub fn same_i32(&self, other: &Self) -> bool {
        self.value == other.value
    }
    pub fn map_index(&self) -> u8 {
        self.map_index
    }
    pub fn map_bit_from_map_index(map_index: u8) -> u32 {
        0x80000000u32 >> map_index
    }
    pub fn to_map_bit(&self) -> u32 {
        Self::map_bit_from_map_index(self.map_index)
    }

    pub fn to_base_index(&self, map: TrieMap) -> usize {
        u32::count_ones(!(0xFFFFFFFFu32 >> self.map_index) & map.0) as usize
    }
}

fn map_index(hash: u32, hash_index: u32) -> u8 {
    ((hash >> 5 * hash_index) & 0x1f) as u8
}

fn hash_key(key: u32, level: u32) -> u32 {
    let key_bytes = key.to_be_bytes() as [u8; 4];
    let level = level as u64;
    let mut a: u128 = 31415;
    const B: u128 = 27183;
    let mut hash: u128 = 0;
    for i in 0..key_bytes.len() {
        hash = a.wrapping_mul(hash).wrapping_mul(level as u128) + key_bytes[i] as u128;
        a = a.wrapping_mul(B);
    }
    hash as u32
}
