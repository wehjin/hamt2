use crate::{Base, HashKey, MapBase, Slot, SlotMap, TrieValue};
use std::sync::Arc;

#[cfg(test)]
pub async fn one_kv(key: HashKey, value: TrieValue) -> MapBase {
    use crate::services::base;
    let base = base::form_kv(key, value).await;
    MapBase {
        map: SlotMap::set_key_bit(key),
        base: Arc::new(base),
    }
}

pub async fn two_kv(
    key: HashKey,
    value: TrieValue,
    key2: HashKey,
    value2: TrieValue,
) -> MapBase {
    debug_assert!(key.i32() != key2.i32());
    debug_assert!(key.map_index() != key2.map_index());
    let map = SlotMap(key.to_map_bit() | key2.to_map_bit());
    let base = {
        let mut slots = Vec::new();
        if key.map_index() < key2.map_index() {
            slots.push(Slot::one_kv(key, value).await);
            slots.push(Slot::one_kv(key2, value2).await);
        } else {
            slots.push(Slot::one_kv(key2, value2).await);
            slots.push(Slot::one_kv(key, value).await);
        }
        Base { slots }
    };
    MapBase {
        map,
        base: Arc::new(base),
    }
}
