use crate::{Base, BufferMut, HashKey, MapBase, Slot, SlotMap, TrieValue};

#[cfg(test)]
pub async fn one_kv<P: BufferMut>(key: HashKey, value: TrieValue, policy: &mut P) -> MapBase {
	use crate::services::base;
	let base = base::form_kv(key, value, policy).await;
    let id = policy.push_base(base).await;
    MapBase {
        map: SlotMap::set_key_bit(key),
        base: id,
    }
}

pub async fn two_kv<P: BufferMut>(
    key: HashKey,
    value: TrieValue,
    key2: HashKey,
    value2: TrieValue,
    policy: &mut P,
) -> MapBase {
    debug_assert!(key.i32() != key2.i32());
    debug_assert!(key.map_index() != key2.map_index());
    let map = SlotMap(key.to_map_bit() | key2.to_map_bit());
    let base = {
        let mut slots = Vec::new();
        if key.map_index() < key2.map_index() {
            slots.push(Slot::one_kv(key, value, policy).await);
            slots.push(Slot::one_kv(key2, value2, policy).await);
        } else {
            slots.push(Slot::one_kv(key2, value2, policy).await);
            slots.push(Slot::one_kv(key, value, policy).await);
        }
        Base { slots }
    };
    let id = policy.push_base(base).await;
    MapBase { map, base: id }
}
