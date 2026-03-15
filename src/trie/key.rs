#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrieKey {
    value: u32,
    hash: u32,
    map_index: u8,
}

impl TrieKey {
    pub fn new(value: u32) -> Self {
        let hash = hash_key(value, 1);
        let map_index = (hash & 0x1f) as u8;
        Self {
            value,
            hash,
            map_index,
        }
    }
    pub fn u32(&self) -> u32 {
        self.value
    }
    pub fn map_index(&self) -> u8 {
        self.map_index
    }
    pub fn to_map_bit(&self) -> u32 {
        0x80000000u32 >> self.map_index
    }
    pub fn to_base_index(&self, map: u32) -> usize {
        u32::count_ones(!(0xFFFFFFFFu32 >> self.map_index) & map) as usize
    }
}

fn hash_key(key: u32, level: u8) -> u32 {
    let key_bytes = key.to_be_bytes() as [u8; 4];
    let level = level as u64;
    let mut a: u64 = 31415;
    const B: u64 = 27183;
    let mut hash: u64 = 0;
    for i in 0..key_bytes.len() {
        hash = a * hash * level + key_bytes[i] as u64;
        a = a.wrapping_mul(B);
    }
    hash as u32
}
