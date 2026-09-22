use crate::storage::StorageHead;
use crate::trie::{MapBase, SlotBaseId, TrieRead};

/// A trait for reading Bases from storage.
///
/// Base id [`SlotBaseId::ZERO`] is reserved and always represents the empty base.
#[allow(async_fn_in_trait)]
pub trait ReadStorage: TrieRead + Sync + Sized {
    /// The storage type of an owned read-only snapshot, produced by
    /// [`ReadStorage::snapshot`]. Writer storages use their read-only
    /// snapshot type; read-only snapshot types usually use `Self`.
    type Snapshot: ReadStorage + TrieRead + Send + Clone;

    /// Returns an owned read-only snapshot of this storage. The snapshot does
    /// not experience writes made after this call.
    fn snapshot(&self) -> Self::Snapshot;

    fn status(&self) -> StorageHead;

    /// Convert the storage into one that reads starting at the new root. The new
    /// root should exist within the existing root's tree.
    fn with_new_root(self, new_root: Option<MapBase>) -> Self;

    /// Get the maximum reading id.
    fn max_id(&self) -> SlotBaseId {
        self.status().max_id
    }
}
