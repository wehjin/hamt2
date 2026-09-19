use crate::shared::remote::{RemoteClient, RemoteClientReadStorage, SpawnTask};
use sky_trie::prelude::ReadStorage;
use sky_trie::types::slot_base::SlotBase;
use sky_types::storage::ReadStorageError;
use sky_types::trie::{MapBase, SlotBaseId};

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.inner.read(id).await
    }

    fn max_id(&self) -> SlotBaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}
