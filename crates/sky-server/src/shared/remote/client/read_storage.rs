use crate::shared::remote::{RemoteClient, RemoteClientReadStorage, SpawnTask};
use sky_types::storage::{ReadStorage, ReadStorageError};
use sky_types::trie::{
    HandleTrieConfig, MapBase, SlotBase, SlotBaseId, TrieQueryError, TrieBaseRead,
};

impl<T: SpawnTask> TrieBaseRead for RemoteClient<T> {
    type Config = HandleTrieConfig;

    async fn read_base(
        &self,
        id: SlotBaseId,
    ) -> Result<SlotBase<HandleTrieConfig>, TrieQueryError> {
        self.inner.read_base(id).await
    }
}

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase<HandleTrieConfig>, ReadStorageError> {
        self.inner.read_base(id).await.map_err(|err| {
            let TrieQueryError::ReadStorage(read_storage_error) = err else {
                unreachable!("inner.read_base() should a ReadStorage variant");
            };
            read_storage_error
        })
    }

    fn max_id(&self) -> SlotBaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase<HandleTrieConfig> {
        self.inner.read_root()
    }
}
