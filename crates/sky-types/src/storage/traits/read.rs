use crate::storage::{FileReadStorage, FileStorage, ReadStorageError, StorageHead, StoreRead};
use crate::trie::{SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError};
/// A trait for reading Bases from storage.
///
/// Base id [`SlotBaseId::ZERO`] is reserved and always represents the empty base.
#[allow(async_fn_in_trait)]
pub trait ReadStorage: StoreRead + TrieBaseRead + Sync {
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
    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.read_base(id).await.map_err(|e| {
            let TrieQueryError::ReadStorage(rs_error) = e else {
                unreachable!("non-read-storage error")
            };
            rs_error
        })
    }

    fn get_head(&self) -> StorageHead {
        self.status().clone()
    }
}

macro_rules! impl_trie_base_read {
    ($storage:ty) => {
        impl TrieBaseRead for $storage {
            async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
                let base = self.read(id).await?;
                Ok(base)
            }
        }
    };
}

impl_trie_base_read!(FileStorage);
impl_trie_base_read!(FileReadStorage);
