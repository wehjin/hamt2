use crate::hamt::trie::key::TrieKey;

pub struct MemTrie {
    pub map_base: MemMapBase,
}

impl MemTrie {
    pub fn empty() -> Self {
        let map_base = MemMapBase::empty();
        Self { map_base }
    }
    pub fn insert(self, key: u32, value: u32) -> Self {
        let key = TrieKey::new(key);
        let map_base = self.map_base.insert_key_value(key, value);
        Self { map_base }
    }
}

pub struct MemMapBase {
    pub map: u32,
    pub base: Vec<MemSlot>,
}

impl MemMapBase {
    pub fn empty() -> Self {
        Self {
            map: 0,
            base: vec![],
        }
    }
    pub fn insert_key_value(self, key: TrieKey, value: u32) -> Self {
        let map_bit = key.to_map_bit();
        assert_eq!(map_bit & self.map, 0);
        let base_index = key.to_base_index(self.map);
        let mut base = self.base;
        base.insert(base_index, MemSlot::KeyValue(key.u32(), value));
        let map = self.map | map_bit;
        Self { map, base }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum MemSlot {
    #[default]
    Empty,
    KeyValue(u32, u32),
}
