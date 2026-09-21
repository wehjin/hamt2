use crate::storage::{
    FileReadStorage, FileStorage, MemReadStorage, MemStorage, ReadStorageError, StorageHead,
};
use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError};
/// A trait for reading Bases from storage.
///
/// Base id [`SlotBaseId::ZERO`] is reserved and always represents the empty base.
#[allow(async_fn_in_trait)]
pub trait ReadStorage: TrieBaseRead + Sync {
    /// The storage type of an owned read-only snapshot, produced by
    /// [`ReadStorage::snapshot`]. Writer storages use their read-only
    /// snapshot type; read-only snapshot types usually use `Self`.
    type Snapshot: ReadStorage + TrieBaseRead + Send + Clone;

    /// Returns an owned read-only snapshot of this storage. The snapshot does
    /// not observe writes made after this call.
    fn snapshot(&self) -> Self::Snapshot;

    /// Reads a base from storage.
    ///
    /// Base id [`SlotBaseId::ZERO`] always reads back the empty base. Reading
    /// any other unwritten id is a programming error: writer storages panic
    /// for ids they have never assigned, and snapshot readers panic for ids
    /// beyond their captured `max_id`. Ids read from committed map bases are
    /// always valid.
    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError>;

    /// Returns the highest base id in the storage. The empty base id
    /// ([`SlotBaseId::ZERO`]) counts, so an empty storage returns
    /// [`SlotBaseId::ZERO`].
    fn max_id(&self) -> SlotBaseId;

    /// Reads the committed root map base, returning [`MapBase::empty()`] when
    /// no root has been committed yet. Every implementation holds the root in
    /// memory, so this never touches the backing medium.
    fn read_root(&self) -> MapBase;

    fn get_head(&self) -> StorageHead {
        let max_id = self.max_id();
        let root = self.read_root();
        StorageHead { max_id, root }
    }
}

macro_rules! impl_trie_base_read {
    ($storage:ty) => {
        impl TrieBaseRead for $storage {
            async fn read_base(
                &self,
                id: SlotBaseId,
            ) -> Result<SlotBase, TrieQueryError> {
                let base = self.read(id).await?;
                Ok(base)
            }
        }
    };
}

impl_trie_base_read!(MemStorage);
impl_trie_base_read!(MemReadStorage);
impl_trie_base_read!(FileStorage);
impl_trie_base_read!(FileReadStorage);
