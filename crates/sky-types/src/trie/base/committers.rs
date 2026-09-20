use crate::trie::{
    HashKey, Slot, SlotBase, TrieConfig, TrieInsertError, TrieValue, TrieWritePolicy, map_base,
};

/// Makes a copy of `base` where the kv already at `index` is moved into a new
/// base containing both the old kv and a new kv.
///
/// #Effects
/// This functions commits sub-bases into the write-policy.
pub async fn kick_kv<C, P>(
    base: SlotBase<C>,
    index: usize,
    key: HashKey,
    value: TrieValue<C>,
    policy: &mut P,
) -> SlotBase<C>
where
    C: TrieConfig,
    P: TrieWritePolicy<Config = C>,
{
    let post_slot = {
        let Slot::KeyValue(b_key, b_value) = base[index].clone() else {
            unreachable!("Should be a key-value slot, not a map-base slot:")
        };
        let b_key = key.sync(b_key);
        debug_assert!(b_key.i32() != key.i32());
        Slot::two_kv(b_key.next(), b_value, key.next(), value, policy).await
    };
    base.replace_slot(index, post_slot)
}

/// Makes a copy of `base` where `key` and `value` are inserted into the
/// sub-base already present at `index`.
///
/// #Effects
/// This functions commits sub-bases into the write-policy.
pub async fn merge_kv<C, P>(
    base: SlotBase<C>,
    index: usize,
    key: HashKey,
    value: TrieValue<C>,
    policy: &mut P,
) -> Result<SlotBase<C>, TrieInsertError>
where
    C: TrieConfig,
    P: TrieWritePolicy<Config = C>,
{
    let pre_slot = base[index].clone();
    let post_slot = {
        let Slot::MapBase(pre_map_base) = pre_slot else {
            unreachable!("Should be a map-base slot, not a key-value slot:")
        };
        let post_map_base = map_base::insert_kv(pre_map_base, key.next(), value, policy).await?;
        Slot::MapBase(post_map_base)
    };
    Ok(base.replace_slot(index, post_slot))
}
