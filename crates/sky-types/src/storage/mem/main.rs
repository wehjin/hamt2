use crate::storage::{MemTrieView, ReadStorageError, StorageStatus, TrieView};
use crate::trie::{Base, BaseId, BaseRead, MapBase};

#[derive(Debug)]
pub struct MemTrie {
    inner: MemTrieView,
}

impl MemTrie {
    pub fn new() -> Self {
        let inner = MemTrieView::empty();
        Self { inner }
    }
}

impl TrieView for MemTrie {
    type Snapshot = MemTrieView;

    fn status(&self) -> StorageStatus {
        self.inner.status()
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl BaseRead for MemTrie {
    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}
