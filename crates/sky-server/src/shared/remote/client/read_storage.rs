use crate::shared::remote::{Remote, RemoteClient, SpawnTask};
use sky_types::storage::ReadStorageError;
use sky_types::trie::TrieSnap;
use sky_types::trie::{Base, BaseId, BaseRead, MapBase};

impl<T: SpawnTask> BaseRead for RemoteClient<T> {
    fn max_id(&self) -> BaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}

impl<T: SpawnTask> TrieSnap for RemoteClient<T> {
    type Snapshot = Remote<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}
