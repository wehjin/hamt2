use crate::storage::{
    BaseStore, MemBaseStore, MemTrieCore, MemTrieView, ReadStorageError, StorageStatus, TrieView,
};
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};

pub fn mem_trie_new() -> MemTrie<MemBaseStore> {
    let store = MemBaseStore::new();
    let trie = MemTrie::load(store);
    trie
}

#[derive(Debug)]
pub struct MemTrie<S: BaseStore> {
    inner: MemTrieView<S>,
}

impl<S: BaseStore + Send + Sync> MemTrie<S> {
    pub fn load(store: S) -> Self {
        let inner = MemTrieView::load(store);
        Self { inner }
    }

    pub async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut MemTrieCore<S>) -> anyhow::Result<Out>,
    {
        let mut edit = MemTrieCore::extend(&self.inner).await?;
        let out = f(&mut edit).await?;
        let committed = edit.commit().await?;
        self.inner = committed.snapshot();
        Ok(out)
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for MemTrie<S> {
    type Subtrie = MemTrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl<S: BaseStore + Send + Sync> TrieView for MemTrie<S> {
    type Snapshot = MemTrieView<S>;

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

impl<S: BaseStore> BaseRead for MemTrie<S> {
    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}
