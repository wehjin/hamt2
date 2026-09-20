use crate::trie::{HashKey, Slot, SlotBase, TrieInsertError, TrieValue, map_base};
use std::fmt::Debug;

#[allow(async_fn_in_trait)]
pub trait TrieWritePolicy: TrieReadPolicy
where
    Self: Sized,
{
    type WriteErrorType: Debug;

    async fn commit_single_slot_base(
        &mut self,
        key: HashKey,
        value: TrieValue<Self::HandleType>,
    ) -> Result<Self::HandleType, Self::WriteErrorType>;

    async fn commit_base(
        &mut self,
        base: Self::BaseType,
    ) -> Result<Self::HandleType, Self::WriteErrorType>;

    fn replace_slot_value_in_base(
        &self,
        base: Self::BaseType,
        index: usize,
        value: TrieValue<Self::HandleType>,
    ) -> Self::BaseType;

    /// Makes a copy of `base` where the kv already at `index` is moved into a new
    /// base containing both the old kv and a new kv.
    async fn kick_kv(
        &mut self,
        base: Self::BaseType,
        base_index: usize,
        key: HashKey,
        value: TrieValue<Self::HandleType>,
    ) -> Self::BaseType {
        let base_ref = base.as_ref();
        let post_slot = {
            let Slot::KeyValue(b_key, b_value) = base_ref[base_index].clone() else {
                unreachable!("Should be a key-value slot, not a map-base slot:")
            };
            let b_key = key.sync(b_key);
            debug_assert!(b_key.i32() != key.i32());
            Slot::two_kv(b_key.next(), b_value, key.next(), value, self).await
        };
        let new_base = base_ref.replace_slot(base_index, post_slot);
        Self::BaseType::from(new_base)
    }

    /// Makes a copy of `base` where a new key and value are inserted in the
    /// map-base already present at `index`.
    async fn merge_kv(
        &mut self,
        base: Self::BaseType,
        base_index: usize,
        key: HashKey,
        value: TrieValue<Self::HandleType>,
    ) -> Result<Self::BaseType, TrieInsertError> {
        let base_ref = base.as_ref();
        let pre_slot = base_ref[base_index].clone();
        let post_slot = {
            let Slot::MapBase(pre_map_base) = pre_slot else {
                unreachable!("Should be a map-base slot, not a key-value slot:")
            };
            let post_map_base = map_base::insert_kv(pre_map_base, key.next(), value, self).await?;
            Slot::MapBase(post_map_base)
        };
        let new_base = base_ref.replace_slot(base_index, post_slot);
        Ok(Self::BaseType::from(new_base))
    }
}

#[allow(async_fn_in_trait)]
pub trait TrieReadPolicy {
    type HandleType: Clone + Eq + PartialEq + Default;
    type BaseType: AsRef<SlotBase<Self::HandleType>> + Clone + From<SlotBase<Self::HandleType>>;
    type ReadErrorType: Debug;

    async fn read_base(&self, id: Self::HandleType) -> Result<Self::BaseType, Self::ReadErrorType>;
}
