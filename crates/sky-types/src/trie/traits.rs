use crate::trie::{SlotBase, TrieConfig, TrieInsertError, TrieQueryError};

#[allow(async_fn_in_trait)]
pub trait TrieBaseCommit: TrieBaseRead
where
    Self: Sized,
{
    /// Commits a base and returns its assigned handle.
    async fn commit_base(
        &mut self,
        base: SlotBase<Self::Config>,
    ) -> Result<<Self::Config as TrieConfig>::HandleType, TrieInsertError>;
}

#[allow(async_fn_in_trait)]
pub trait TrieBaseRead {
    type Config: TrieConfig;

    async fn read_base(
        &self,
        id: <Self::Config as TrieConfig>::HandleType,
    ) -> Result<SlotBase<Self::Config>, TrieQueryError>;
}
