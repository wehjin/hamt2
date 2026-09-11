use crate::trie::base::{Base, BaseId};
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::slot::MemSlot;
use crate::trie::mem::value::MemValue;

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: TrieMap::empty(),
            base: BaseId(0),
        }
    }

    pub async fn one_kv(
        key: TrieKey,
        value: MemValue,
        storage: &mut impl BaseStorageReadWrite,
    ) -> Self {
        let id = storage.append(&Base::new_kv(key, value)).await.expect("append base");
        Self {
            map: TrieMap::set_key_bit(key),
            base: id,
        }
    }
    pub async fn two_kv(
        key: TrieKey,
        value: MemValue,
        key2: TrieKey,
        value2: MemValue,
        storage: &mut impl BaseStorageReadWrite,
    ) -> Self {
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
        let id = storage.append(&base).await.expect("append base");
        MapBase { map, base: id }
    }
}
