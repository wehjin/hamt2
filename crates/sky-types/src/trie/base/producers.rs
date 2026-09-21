use crate::trie::{HashKey, Slot, SlotBase, TrieValue};

/// Makes a new base containing `key` and `value` in a single slot.
pub fn form_kv(key: HashKey, value: TrieValue) -> SlotBase {
    SlotBase::new_kv(key, value)
}

/// Makes a copy of `base` in a new slot containing `key` and `value` are inserted
/// at `index`.
pub fn insert_kv(
    base: SlotBase,
    index: usize,
    key: HashKey,
    value: TrieValue,
) -> SlotBase {
    let slot = Slot::one_kv(key, value);
    base.as_ref().insert_slot(index, slot)
}

/// Makes a copy of `base` in which the slot at `index` contains `value` in place
/// of its previous value while preserving the key.
pub fn swap_v(
    base: SlotBase,
    index: usize,
    value: TrieValue,
) -> SlotBase {
    let SlotBase { mut slots } = base;
    let revised_slot = slots.remove(index).replace_value(value);
    slots.insert(index, revised_slot);
    SlotBase { slots }
}
