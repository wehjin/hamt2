use crate::trie::{SlotBase, SlotBaseId, TrieInsertError, TrieRead};
#[allow(async_fn_in_trait)]
pub trait TrieCommit: TrieRead
where
    Self: Sized,
{
    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: SlotBase) -> Result<SlotBaseId, TrieInsertError>;
}
