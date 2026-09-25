use crate::shared::remote::{Remote, RemoteClient, SpawnTask};
use sky_types::storage::{ReadStorageError, TrieView};
use sky_types::trie::BaseView;
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

impl<T: SpawnTask> BaseView for RemoteClient<T> {
    type Snapshot = TrieView<Remote<T>>;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}
