use crate::types::slot_base::SlotBase;
use errors::{StorageReadError, StorageWriteError};
use sky_types::trie::SlotBaseId;
use sky_types::trie::MapBase
;

pub mod errors;
pub mod file;
pub mod mem;

/// A trait for reading Bases from storage.
///
/// Base id 0 is reserved and always represents the empty base.
pub trait ReadTrieStorage: Sync {
    /// The storage type of an owned read-only snapshot, produced by
    /// [`ReadTrieStorage::snapshot`]. Writer storages use their read-only
    /// snapshot type; read-only snapshot types usually use `Self`.
    type Snapshot: ReadTrieStorage + Send + Clone;

    /// Returns an owned read-only snapshot of this storage. The snapshot does
    /// not observe writes made after this call.
    fn snapshot(&self) -> Self::Snapshot;

    /// Reads a base from storage.
    fn read(
        &self,
        id: SlotBaseId,
    ) -> impl Future<Output = Result<SlotBase, StorageReadError>> + Send;

    /// Returns the highest base id in the storage or none if empty. Base id 0 (the empty base) is not counted.
    fn max_id(&self) -> Option<SlotBaseId>;

    /// Reads the root map base of the trie or none if no root has been committed.
    fn read_root(
        &self,
    ) -> impl Future<Output = Result<Option<MapBase>, StorageReadError>> + Send;

    /// Reads the root map base of the trie, defaulting to the empty map base if
    /// no root has been committed.
    fn get_root(&self) -> impl Future<Output = Result<MapBase, StorageReadError>> + Send {
        async { Ok(self.read_root().await?.unwrap_or_else(|| MapBase::empty())) }
    }
}

/// A trait for reading and writing Bases from storage.
pub trait ReadWriteTrieStorage: ReadTrieStorage {
    /// Read the next available base id. The value is 1 in an empty storage because base id 0 is reserved for the empty base.
    fn next_id(&self) -> SlotBaseId;

    /// Stores a base and assigns it the next available id. The id can be used to read back the base in `BaseStorageRead::read`.
    fn append(
        &mut self,
        base: &SlotBase,
    ) -> impl Future<Output = Result<SlotBaseId, StorageWriteError>> + Send;

    /// Persists the given root map base. The root can be read back with `BaseStorageRead::read_root`.
    fn write_root(
        &mut self,
        root: MapBase,
    ) -> impl Future<Output = Result<(), StorageWriteError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie_storage::mem::MemTrieStorage;
    use crate::types::HashKey;
    use sky_types::trie::TrieValue
;

    #[tokio::test]
    async fn empty_storage_has_no_ids() {
        let storage = MemTrieStorage::new();
        assert_eq!(None, storage.max_id());
        assert_eq!(SlotBaseId(1), storage.next_id());
    }

    #[tokio::test]
    async fn base_id_zero_is_the_empty_base() {
        let storage = MemTrieStorage::new();
        assert_eq!(
            SlotBase::new(),
            storage.read(SlotBaseId(0)).await.expect("read")
        );
    }

    #[tokio::test]
    async fn append_assigns_sequential_ids() {
        let mut storage = MemTrieStorage::new();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = storage.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(1), id0);
        assert_eq!(SlotBaseId(2), id1);
        assert_eq!(Some(SlotBaseId(2)), storage.max_id());
        assert_eq!(SlotBaseId(3), storage.next_id());
        assert_eq!(base, storage.read(id0).await.expect("read"));
        assert_eq!(base, storage.read(id1).await.expect("read"));
    }

    #[tokio::test]
    async fn mem_readonly_snapshot_does_not_see_new_bases() {
        let mut storage = MemTrieStorage::new();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id = storage.append(&base).await.expect("append");
        let view = storage.snapshot();
        storage.append(&base).await.expect("append");
        assert_eq!(Some(SlotBaseId(2)), storage.max_id());
        assert_eq!(Some(id), view.max_id());
        assert_eq!(base, view.read(id).await.expect("read"));
    }
}
