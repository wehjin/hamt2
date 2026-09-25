use crate::storage::{
    BaseStore, MemBaseStore, MemTrieView, ReadStorageError, StorageStatus, BaseEdit, BaseView,
    WriteStorageError,
};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase, TrieStream};
use std::sync::Arc;

pub fn mem_edit_new() -> MemTrieEdit<MemBaseStore> {
    let store = MemBaseStore::new();
    let edit = MemTrieEdit::load(store);
    edit
}

#[derive(Debug)]
pub struct MemTrieEdit<S: BaseStore> {
    past: Option<Arc<MemTrieView<S>>>,
    store: S,
}

impl<S: BaseStore> MemTrieEdit<S> {
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

impl<S: BaseStore + Send + Sync> BaseEdit for MemTrieEdit<S> {}

impl<S: BaseStore> BaseCommit for MemTrieEdit<S> {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.store.set_root(root).await
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        self.store.push_base(base).await
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for MemTrieEdit<S> {
    type Subtrie = MemTrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.snapshot().to_subtrie(subtrie_root)
    }
}

impl<S: BaseStore + Send + Sync> BaseView for MemTrieEdit<S> {
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

impl<S: BaseStore> BaseRead for MemTrieEdit<S> {
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

#[cfg(test)]
mod tests {
    use crate::storage::mem_edit_new;
    use crate::trie::{TrieInsert, TrieQuery, TrieStream, TrieValue};
    use futures::StreamExt;

    #[tokio::test]
    async fn query_value_exists_works() {
        let mut m = mem_edit_new();
        m.insert(32, TrieValue::U32(33)).await.unwrap();
        let v = m.query(32).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(33)));
    }

    #[tokio::test]
    async fn u32_stream_exists_works() {
        let mut m = mem_edit_new();
        m.insert(27, TrieValue::U32(28)).await.unwrap();
        let v = m.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(v, vec![(27, 28)]);
    }

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut m = mem_edit_new();
        m.deep_insert([1, 2, 3], 45, false).await.unwrap();
        let v = m.deep_query([1, 2, 3]).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(45)));
    }
}
