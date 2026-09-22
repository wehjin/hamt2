use crate::trie::{Base, BaseId, TrieInsertError, RootBaseRead};
#[allow(async_fn_in_trait)]
pub trait TrieCommit: RootBaseRead
where
    Self: Sized,
{
    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: Base) -> Result<BaseId, TrieInsertError>;
}
