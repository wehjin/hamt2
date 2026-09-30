use crate::storage::WriteStorageError;
use crate::trie::{Base, Buffer, BufferIndex, MapBase};
#[allow(async_fn_in_trait)]
pub trait BufferMut: Buffer
where
    Self: Sized,
{
    /// Commits a new `root` into the trie.
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError>;

    /// Commits a base and returns its assigned handle.
    async fn commit_base(&mut self, base: Base) -> Result<BufferIndex, WriteStorageError>;
}
