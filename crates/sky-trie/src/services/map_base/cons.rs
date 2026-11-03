use crate::{Base, HashKey, MapBase, Slot, SlotMap, TrieValue};
use std::sync::Arc;

pub fn two_kv(
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
            slots.push(Slot::one_kv(key, value));
            slots.push(Slot::one_kv(key2, value2));
        } else {
            slots.push(Slot::one_kv(key2, value2));
            slots.push(Slot::one_kv(key, value));
        }
        Base { slots }
    };
    MapBase {
        map,
        base: Arc::new(base),
    }
}
