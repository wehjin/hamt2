use crate::storage::WriteStorageError;
use crate::trie::{Base, BaseId, BaseRead, MapBase};
#[allow(async_fn_in_trait)]
pub trait BaseCommit: BaseRead
where
    Self: Sized,
{
    /// Read the next available base id.
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }

    /// Commits a new `root` into the trie.
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError>;

    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError>;
}
