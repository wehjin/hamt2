use crate::storage::{MemTrieEdit, MemTrieView, ReadStorageError, StorageStatus, TrieView};
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};

#[derive(Debug)]
pub struct MemTrie {
    inner: MemTrieView,
}

impl MemTrie {
    pub async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut MemTrieEdit) -> anyhow::Result<Out>,
    {
        let mut edit = MemTrieEdit::extend(&self.inner.clone());
        let result = f(&mut edit).await;
        if let Ok(out) = result {
            self.inner.merge(edit.inner).await;
            Ok(out)
        } else {
            result
        }
    }

    pub fn new() -> Self {
        let inner = MemTrieView::empty();
        Self { inner }
    }
}

impl TrieStream for MemTrie {
    type Subtrie = MemTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.snapshot().to_subtrie(subtrie_root)
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
