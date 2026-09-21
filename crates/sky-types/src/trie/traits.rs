use crate::trie::{SlotBase, SlotBaseId, TrieInsertError, TrieQueryError};

#[allow(async_fn_in_trait)]
pub trait TrieBaseCommit: TrieBaseRead
where
    Self: Sized,
{
    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: SlotBase) -> Result<SlotBaseId, TrieInsertError>;
}

#[allow(async_fn_in_trait)]
pub trait TrieBaseRead {
    async fn read_base(
        &self,
        id: SlotBaseId,
    ) -> Result<SlotBase, TrieQueryError>;
}
