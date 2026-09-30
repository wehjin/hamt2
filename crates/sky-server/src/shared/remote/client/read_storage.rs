use crate::shared::remote::{Remote, RemoteClient, SpawnTask};
use sky_types::storage::ReadStorageError;
use sky_types::trie::TrieSnap;
use sky_types::trie::{Base, BufferIndex, Buffer, MapBase};

impl<T: SpawnTask> Buffer for RemoteClient<T> {
    fn max_index(&self) -> BufferIndex {
        self.inner.max_index()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn get_base(&self, id: BufferIndex) -> Result<Base, ReadStorageError> {
        self.inner.get_base(id).await
    }
}

impl<T: SpawnTask> TrieSnap for RemoteClient<T> {
    type Snapshot = Remote<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}
