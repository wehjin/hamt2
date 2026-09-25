use crate::storage::{
    BaseStore, MemTrieView, ReadStorageError, StorageStatus, TrieEdit, TrieView, WriteStorageError,
};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase, TrieStream};
use std::sync::Arc;

#[derive(Debug)]
pub struct MemTrieCore<S: BaseStore> {
    past: Option<Arc<MemTrieView<S>>>,
    store: S,
}

impl<S: BaseStore> MemTrieCore<S> {
    pub(crate) fn load(store: S) -> Self {
        Self { past: None, store }
    }
    pub(crate) async fn extend(past: &MemTrieView<S>) -> Result<Self, WriteStorageError> {
        let extension = Self {
            past: Some(Arc::new(past.clone())),
            store: past.store.extend().await?,
        };
        Ok(extension)
    }
    pub(crate) async fn commit(self) -> Result<Self, WriteStorageError> {
        let Some(past) = self.past else {
            panic!("cannot merge with a past");
        };
        let merged = Self {
            past: past.past.clone(),
            store: self.store.commit(&past.store).await?,
        };
        Ok(merged)
    }
}

impl<S: BaseStore + Send + Sync> TrieEdit for MemTrieCore<S> {}

impl<S: BaseStore> BaseCommit for MemTrieCore<S> {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.store.set_root(root).await
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        self.store.push_base(base).await
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for MemTrieCore<S> {
    type Subtrie = MemTrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.snapshot().to_subtrie(subtrie_root)
    }
}

impl<S: BaseStore + Send + Sync> TrieView for MemTrieCore<S> {
    type Snapshot = MemTrieView<S>;

    fn status(&self) -> StorageStatus {
        StorageStatus {
            max_id: self.store.max_id(),
            root: self.store.root(),
        }
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let Self { past, store } = self;
        let store = store.with_root(new_root);
        Self { past, store }
    }

    fn snapshot(&self) -> Self::Snapshot {
        MemTrieView {
            past: self.past.clone(),
            store: Arc::new(self.store.snapshot()),
        }
    }
}

impl<S: BaseStore> BaseRead for MemTrieCore<S> {
    fn read_root(&self) -> MapBase {
        self.store.root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id >= self.store.start_id() {
            self.store.base(id).await
        } else if let Some(past) = self.past.as_ref() {
            past.read_base(id).await
        } else {
            Ok(Base::empty())
        }
    }
}
