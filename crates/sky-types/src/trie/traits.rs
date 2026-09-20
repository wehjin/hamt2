use crate::trie::{SlotBase, TrieConfig};
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
