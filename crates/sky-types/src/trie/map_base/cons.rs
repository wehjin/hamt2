use crate::trie::{
	HashKey, MapBase, Slot, Base, SlotMap, TrieInsertError, TrieValue, TrieCommit,
};

#[cfg(test)]
pub async fn one_kv<P: TrieCommit>(
    key: HashKey,
    value: TrieValue,
    policy: &mut P,
) -> Result<MapBase, TrieInsertError> {
    use crate::trie::base;
    let base = base::form_kv(key, value);
    let id = policy.commit_base(base).await?;
    let map_base = MapBase {
        map: SlotMap::set_key_bit(key),
        base: id,
    };
    Ok(map_base)
}

pub async fn two_kv<P: TrieCommit>(
    key: HashKey,
    value: TrieValue,
    key2: HashKey,
    value2: TrieValue,
    policy: &mut P,
) -> Result<MapBase, TrieInsertError> {
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
    let id = policy.commit_base(base).await?;
    Ok(MapBase { map, base: id })
}
