use crate::shared::remote::{RemoteClient, RemoteClientReadStorage, SpawnTask};
use sky_types::storage::{ReadStorage, ReadStorageError};
use sky_types::trie::{MapBase, SlotBase, SlotBaseId, TrieReadPolicy};

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase<SlotBaseId>, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn max_id(&self) -> SlotBaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase<SlotBaseId> {
        self.inner.read_root()
    }
}
