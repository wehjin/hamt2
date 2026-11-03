use crate::{HashKey, MapBase, TrieValue};

pub fn query_value(map_base: MapBase, key: HashKey) -> Option<TrieValue> {
    let MapBase { map, base } = map_base;
    match map.try_base_index(key) {
        Some(base_index) => base.as_ref()[base_index].query_value(key),
        None => None,
    }
}

pub fn query_keys_values(map_base: MapBase) -> Vec<(i32, TrieValue)> {
    let MapBase { map, base } = map_base;
    let mut out = Vec::new();
    let slot_count = map.slot_count();
    let base_ref = base.as_ref();
    debug_assert_eq!(slot_count, base_ref.len());
    for base_index in 0..slot_count {
        let keys_values = base_ref[base_index].query_key_values();
        out.extend(keys_values);
    }
    out
}
