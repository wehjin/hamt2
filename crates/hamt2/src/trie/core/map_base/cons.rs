use crate::trie::core::key::TrieKey;
use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::base::Base;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;

impl TrieMapBase {
    pub fn empty() -> Self {
        let map = TrieMap::empty();
        let base = Base::new();
        Self { map, base }
    }

    pub fn one_kv(key: TrieKey, value: MemValue) -> Self {
        let map = TrieMap::set_key_bit(key);
        let base = Base::new_kv(key, value);
        Self { map, base }
    }
    pub fn two_kv(key: TrieKey, value: MemValue, key2: TrieKey, value2: MemValue) -> Self {
        debug_assert!(key.i32() != key2.i32());
        debug_assert!(key.map_index() != key2.map_index());
        let map = TrieMap(key.to_map_bit() | key2.to_map_bit());
        let base = {
            let mut slots = Vec::new();
            if key.map_index() < key2.map_index() {
                slots.push(MemSlot::one_kv(key, value));
                slots.push(MemSlot::one_kv(key2, value2));
            } else {
                slots.push(MemSlot::one_kv(key2, value2));
                slots.push(MemSlot::one_kv(key, value));
            }
            let base = Base { slots };
            base
        };
        TrieMapBase { map, base }
    }
}