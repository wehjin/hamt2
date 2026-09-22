use crate::storage::{ReadStorage, WriteStorageError};
use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieCommit, TrieInsertError};

/// A trait for reading and writing Bases from storage.
#[allow(async_fn_in_trait)]
pub trait ReadWriteStorage: ReadStorage {
    /// Read the next available base id. The value is 1 in an empty storage because base id 0 is reserved for the empty base.
    fn next_id(&self) -> SlotBaseId {
        self.max_id() + 1
    }

    /// Stores a base and assigns it the next available id. The id can be used to read back the base in `BaseStorageRead::read`.
    async fn append(
        &mut self,
        base: &SlotBase,
    ) -> Result<SlotBaseId, WriteStorageError>;

    /// Persists the given root map base. The root can be read back with `BaseStorageRead::read_root`.
    async fn write_root(&mut self, root: MapBase) -> Result<(), WriteStorageError>;
}

impl<T> TrieCommit for T
where
    T: ReadWriteStorage,
{
    async fn commit_base(
        &mut self,
        base: SlotBase,
    ) -> Result<SlotBaseId, TrieInsertError> {
        let id = self.append(&base).await?;
        Ok(id)
    }
}
