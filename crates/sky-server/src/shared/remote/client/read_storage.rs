use crate::shared::remote::{RemoteClient, RemoteClientReadStorage, SpawnTask};
use sky_types::storage::{ReadStorage, ReadStorageError, StorageStatus};
use sky_types::trie::{MapBase, Base, BaseId, RootBaseRead};

impl<T: SpawnTask> RootBaseRead for RemoteClient<T> {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn status(&self) -> StorageStatus {
        self.inner.status()
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}
