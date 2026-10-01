use crate::trie::{Base, BufferMut, HashKey, Slot, TrieInsertError, TrieValue};

/// Makes a new base containing `key` and `value` in a single slot.
pub async fn form_kv(
    key: HashKey,
    value: TrieValue,
    buffer: &mut impl BufferMut,
) -> Result<Base, TrieInsertError> {
    let base = Base::new_kv(key, value, buffer).await?;
    Ok(base)
}

/// Makes a copy of `base` in a new slot containing `key` and `value` are inserted
/// at `index`.
pub async fn insert_kv(
    base: Base,
    index: usize,
    key: HashKey,
    value: TrieValue,
    buffer: &mut impl BufferMut,
) -> Result<Base, TrieInsertError> {
    let slot = Slot::one_kv(key, value, buffer).await?;
    let base = base.as_ref().insert_slot(index, slot);
    Ok(base)
}

/// Makes a copy of `base` in which the slot at `index` contains `value` in place
/// of its previous value while preserving the key.
pub async fn swap_v(
    base: Base,
    index: usize,
    value: TrieValue,
    buffer: &mut impl BufferMut,
) -> Result<Base, TrieInsertError> {
    let Base { mut slots } = base;
    let revised_slot = slots.remove(index).replace_value(value, buffer).await?;
    slots.insert(index, revised_slot);
    let base = Base { slots };
    Ok(base)
}
