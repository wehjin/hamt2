use crate::trie_storage::ReadWriteTrieStorage;
use crate::types::slot_base::SlotBase;
use crate::types::slot_base_id::SlotBaseId;
use crate::types::HashKey;
use crate::types::map_base::MapBase;
use crate::types::slot::Slot;
use crate::types::slot_map::SlotMap;
use crate::types::trie_value::TrieValue;

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: SlotBaseId(0),
        }
    }

    pub async fn one_kv(
        key: HashKey,
        value: TrieValue,
        storage: &mut impl ReadWriteTrieStorage,
    ) -> Self {
        let id = storage.append(&SlotBase::new_kv(key, value)).await.expect("append base");
        Self {
            map: SlotMap::set_key_bit(key),
            base: id,
        }
    }
    pub async fn two_kv(
        key: HashKey,
        value: TrieValue,
        key2: HashKey,
        value2: TrieValue,
        storage: &mut impl ReadWriteTrieStorage,
    ) -> Self {
        debug_assert!(key.i32() != key2.i32());
        debug_assert!(key.map_index() != key2.map_index());
        let map = SlotMap(key.to_map_bit() | key2.to_map_bit());
        let base = {
            let mut slots = Vec::new();
            if key.map_index() < key2.map_index() {
                slots.push(Slot::one_kv(key, value));
                slots.push(Slot::one_kv(key2, value2));
            } else {
                slots.push(Slot::one_kv(key2, value2));
                slots.push(Slot::one_kv(key, value));
            }
            let base = SlotBase { slots };
            base
        };
        let id = storage.append(&base).await.expect("append base");
        MapBase { map, base: id }
    }
}
