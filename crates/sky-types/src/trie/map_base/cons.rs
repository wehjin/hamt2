use crate::trie::{HashKey, MapBase, Slot, SlotBase, SlotMap, TrieValue, TrieWritePolicy};

#[cfg(test)]
pub async fn one_kv<P: TrieWritePolicy>(
    key: HashKey,
    value: TrieValue<P::Config>,
    policy: &mut P,
) -> MapBase<P::Config> {
    let id = policy
        .commit_single_slot_base(key, value)
        .await
        .expect("append base");
    MapBase {
        map: SlotMap::set_key_bit(key),
        base: id,
    }
}

pub async fn two_kv<P: TrieWritePolicy>(
    key: HashKey,
    value: TrieValue<P::Config>,
    key2: HashKey,
    value2: TrieValue<P::Config>,
    policy: &mut P,
) -> MapBase<P::Config> {
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
        SlotBase { slots }
    };
    let id = policy.commit_base(base).await.expect("append base");
    MapBase { map, base: id }
}
