use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieInsertError, TrieQueryError};

#[allow(async_fn_in_trait)]
pub trait TrieBaseCommit: TrieRead
where
    Self: Sized,
{
    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: SlotBase) -> Result<SlotBaseId, TrieInsertError>;
}

#[allow(async_fn_in_trait)]
pub trait TrieRead {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError>;
    fn read_root(&self) -> MapBase;
}
