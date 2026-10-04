use crate::{Buffer, HashKey, MapBase, TrieValue};

pub fn query_value<S: Buffer>(
    map_base: MapBase,
    key: HashKey,
    base_read: &S,
) -> Option<TrieValue> {
    let MapBase { map, base: base_id } = map_base;
    let value = match map.try_base_index(key) {
        Some(base_index) => {
            let base = base_read.get_base(base_id, map.slot_count());
            base.as_ref()[base_index].query_value(key, base_read)
        }
        None => None,
    };
    value
}

pub fn query_keys_values<P: Buffer>(map_base: MapBase, storage: &P) -> Vec<(i32, TrieValue)> {
    let MapBase { map, base: base_id } = map_base;
    let mut out = Vec::new();
    let slot_count = map.slot_count();
    let base = storage.get_base(base_id, map.slot_count());
    let base_ref = base.as_ref();
    debug_assert_eq!(slot_count, base_ref.len());
    for base_index in 0..slot_count {
        let keys_values = base_ref[base_index].query_key_values(storage);
        out.extend(keys_values);
    }
    out
}
