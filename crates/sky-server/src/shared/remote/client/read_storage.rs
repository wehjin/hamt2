use crate::shared::remote::{Remote, RemoteClient, SpawnTask};
use sky_types::trie::TrieSnap;
use sky_types::trie::{Base, Buffer, BufferIndex, MapBase};

impl<T: SpawnTask> Buffer for RemoteClient<T> {
    fn max_index(&self) -> BufferIndex {
        self.inner.max_index()
    }

    fn get_root(&self) -> MapBase {
        self.inner.get_root()
    }

    async fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        self.inner.get_base(id, size).await
    }
}

impl<T: SpawnTask> TrieSnap for RemoteClient<T> {
    type Snapshot = Remote<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}
