use crate::trie::{HashKey, Slot, SlotBase, TrieConfig, TrieValue};

/// Makes a new base containing `key` and `value` in a single slot.
pub fn form_kv<C: TrieConfig>(key: HashKey, value: TrieValue<C>) -> SlotBase<C> {
    SlotBase::new_kv(key, value)
}

/// Makes a copy of `base` in a new slot containing `key` and `value` are inserted
/// at `index`.
pub fn insert_kv<C: TrieConfig>(
    base: SlotBase<C>,
    index: usize,
    key: HashKey,
    value: TrieValue<C>,
) -> SlotBase<C> {
    let slot = Slot::one_kv(key, value);
    base.as_ref().insert_slot(index, slot)
}

/// Makes a copy of `base` in which the slot at `index` contains `value` in place
/// of its previous value while preserving the key.
pub fn swap_v<C: TrieConfig>(
    base: SlotBase<C>,
    index: usize,
    value: TrieValue<C>,
) -> SlotBase<C> {
    let SlotBase { mut slots } = base;
    let revised_slot = slots.remove(index).replace_value(value);
    slots.insert(index, revised_slot);
    SlotBase::<C> { slots }
}
