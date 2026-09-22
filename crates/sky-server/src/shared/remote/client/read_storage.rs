use crate::shared::remote::{RemoteClient, RemoteClientReadStorage, SpawnTask};
use sky_types::storage::{ReadStorage, ReadStorageError, StorageHead, StoreRead};
use sky_types::trie::{MapBase, SlotBase, SlotBaseId, TrieQueryError, TrieRead};

impl<T: SpawnTask> TrieRead for RemoteClient<T> {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}

impl<T: SpawnTask> StoreRead for RemoteClient<T> {
    fn status(&self) -> StorageHead {
        self.inner.status()
    }
}

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.inner.read_base(id).await.map_err(|err| {
            let TrieQueryError::ReadStorage(read_storage_error) = err else {
                unreachable!("inner.read_base() should a ReadStorage variant");
            };
            read_storage_error
        })
    }
}
