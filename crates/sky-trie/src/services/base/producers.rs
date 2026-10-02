use crate::{Base, BufferMut, HashKey, Slot, TrieValue};

/// Makes a new base containing `key` and `value` in a single slot.
pub async fn form_kv(key: HashKey, value: TrieValue, buffer: &mut impl BufferMut) -> Base {
    Base::new_kv(key, value, buffer).await
}

/// Makes a copy of `base` in a new slot containing `key` and `value` are inserted
/// at `index`.
pub async fn insert_kv(
    base: Base,
    index: usize,
    key: HashKey,
    value: TrieValue,
    buffer: &mut impl BufferMut,
) -> Base {
    let slot = Slot::one_kv(key, value, buffer).await;
    base.as_ref().insert_slot(index, slot)
}

/// Makes a copy of `base` in which the slot at `index` contains `value` in place
/// of its previous value while preserving the key.
pub async fn swap_v(
    base: Base,
    index: usize,
    value: TrieValue,
    buffer: &mut impl BufferMut,
) -> Base {
    let Base { mut slots } = base;
    let revised_slot = slots.remove(index).replace_value(value, buffer).await;
    slots.insert(index, revised_slot);
    Base { slots }
}
