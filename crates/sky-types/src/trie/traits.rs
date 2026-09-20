use crate::trie::{HashKey, Slot, SlotBase, TrieConfig, TrieInsertError, TrieValue, map_base};
use std::fmt::Debug;

#[allow(async_fn_in_trait)]
pub trait TrieWritePolicy: TrieReadPolicy
where
    Self: Sized,
{
    type WriteErrorType: Debug;

    /// Commits a base and returns its assigned handle.
    async fn commit_base(
        &mut self,
        base: SlotBase<Self::Config>,
    ) -> Result<<Self::Config as TrieConfig>::HandleType, Self::WriteErrorType>;

    /// Makes a new base containing `key` and `value` in a single slot.
    fn form_kv(&self, key: HashKey, value: TrieValue<Self::Config>) -> SlotBase<Self::Config> {
        SlotBase::new_kv(key, value)
    }

    /// Makes a copy of `base` in which the slot at `index` contains `value` in place
    /// of its previous value while preserving the key.
    fn swap_v(
        &self,
        base: SlotBase<Self::Config>,
        index: usize,
        value: TrieValue<Self::Config>,
    ) -> SlotBase<Self::Config> {
        SlotBase::replace_value(base, index, value)
    }

    /// Makes a copy of `base` where the kv already at `index` is moved into a new
    /// base containing both the old kv and a new kv.
    async fn kick_kv(
        &mut self,
        base: SlotBase<Self::Config>,
        base_index: usize,
        key: HashKey,
        value: TrieValue<Self::Config>,
    ) -> SlotBase<Self::Config> {
        let post_slot = {
            let Slot::KeyValue(b_key, b_value) = base[base_index].clone() else {
                unreachable!("Should be a key-value slot, not a map-base slot:")
            };
            let b_key = key.sync(b_key);
            debug_assert!(b_key.i32() != key.i32());
            Slot::two_kv(b_key.next(), b_value, key.next(), value, self).await
        };
        base.replace_slot(base_index, post_slot)
    }

    /// Makes a copy of `base` where `key` and `value` are inserted into the
    /// lower base already present at `index`.
    async fn merge_kv(
        &mut self,
        base: SlotBase<Self::Config>,
        base_index: usize,
        key: HashKey,
        value: TrieValue<Self::Config>,
    ) -> Result<SlotBase<Self::Config>, TrieInsertError> {
        let pre_slot = base[base_index].clone();
        let post_slot = {
            let Slot::MapBase(pre_map_base) = pre_slot else {
                unreachable!("Should be a map-base slot, not a key-value slot:")
            };
            let post_map_base = map_base::insert_kv(pre_map_base, key.next(), value, self).await?;
            Slot::MapBase(post_map_base)
        };
        Ok(base.replace_slot(base_index, post_slot))
    }
}

#[allow(async_fn_in_trait)]
pub trait TrieReadPolicy {
    type Config: TrieConfig;
    type ReadErrorType: Debug;

    async fn read_base(
        &self,
        id: <Self::Config as TrieConfig>::HandleType,
    ) -> Result<SlotBase<Self::Config>, Self::ReadErrorType>;
}
