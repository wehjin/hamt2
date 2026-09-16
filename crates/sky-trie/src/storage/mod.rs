use crate::types::StorageHead;
use crate::types::slot_base::SlotBase;
use sky_types::storage::error::{StorageReadError, StorageWriteError};
use sky_types::trie::MapBase;
use sky_types::trie::SlotBaseId;

pub mod file;
pub mod mem;

/// A trait for reading Bases from storage.
///
/// Base id [`SlotBaseId::ZERO`] is reserved and always represents the empty base.
pub trait ReadStorage: Sync {
    /// The storage type of an owned read-only snapshot, produced by
    /// [`ReadStorage::snapshot`]. Writer storages use their read-only
    /// snapshot type; read-only snapshot types usually use `Self`.
    type Snapshot: ReadStorage + Send + Clone;

    /// Returns an owned read-only snapshot of this storage. The snapshot does
    /// not observe writes made after this call.
    fn snapshot(&self) -> Self::Snapshot;

    /// Reads a base from storage.
    fn read(
        &self,
        id: SlotBaseId,
    ) -> impl Future<Output = Result<SlotBase, StorageReadError>> + Send;

    /// Returns the highest base id in the storage. The empty base id
    /// ([`SlotBaseId::ZERO`]) counts, so an empty storage returns
    /// [`SlotBaseId::ZERO`].
    fn max_id(&self) -> SlotBaseId;

    /// Reads the committed root map base, returning [`MapBase::empty()`] when
    /// no root has been committed yet.
    fn read_root(&self) -> impl Future<Output = Result<MapBase, StorageReadError>> + Send;

    #[allow(async_fn_in_trait)]
    async fn get_head(&self) -> StorageHead {
        let max_id = self.max_id();
        let root = self.read_root().await.expect("storage reads root");
        StorageHead { max_id, root }
    }
}

/// A trait for reading and writing Bases from storage.
pub trait ReadWriteStorage: ReadStorage {
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
    use crate::crate_services::map_base::one_kv;
    use crate::storage::mem::MemStorage;
    use crate::types::HashKey;
    use sky_types::trie::{MapBase, TrieValue};

    #[tokio::test]
    async fn empty_storage_max_id_is_zero() {
        let storage = MemStorage::new();
        assert_eq!(SlotBaseId::ZERO, storage.max_id());
        assert_eq!(SlotBaseId(1), storage.next_id());
    }

    #[tokio::test]
    async fn base_id_zero_is_the_empty_base() {
        let storage = MemStorage::new();
        assert_eq!(
            SlotBase::new(),
            storage.read(SlotBaseId::ZERO).await.expect("read")
        );
    }

    #[tokio::test]
    async fn empty_storage_root_is_empty() {
        let storage = MemStorage::new();
        assert_eq!(
            MapBase::empty(),
            storage.read_root().await.expect("read root")
        );
    }

    #[tokio::test]
    async fn root_round_trip_works() {
        let mut storage = MemStorage::new();
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage).await;
        storage.write_root(root).await.expect("write root");
        let view = storage.snapshot();
        assert_eq!(root, storage.read_root().await.expect("read root"));
        assert_eq!(root, view.read_root().await.expect("read root"));
    }

    #[tokio::test]
    async fn append_assigns_sequential_ids() {
        let mut storage = MemStorage::new();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = storage.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(1), id0);
        assert_eq!(SlotBaseId(2), id1);
        assert_eq!(SlotBaseId(2), storage.max_id());
        assert_eq!(SlotBaseId(3), storage.next_id());
        assert_eq!(base, storage.read(id0).await.expect("read"));
        assert_eq!(base, storage.read(id1).await.expect("read"));
    }

    #[tokio::test]
    async fn mem_readonly_snapshot_does_not_see_new_bases() {
        let mut storage = MemStorage::new();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id = storage.append(&base).await.expect("append");
        let view = storage.snapshot();
        storage.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(2), storage.max_id());
        assert_eq!(id, view.max_id());
        assert_eq!(base, view.read(id).await.expect("read"));
    }
}
