use crate::storage::traits::BaseStore;
use crate::storage::{ReadStorageError, StorageStatus};
use crate::trie::BaseView;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};
use std::sync::Arc;

#[derive(Debug)]
pub struct TrieView<S: BaseStore> {
    pub(crate) past: Option<Arc<TrieView<S>>>,
    pub(crate) store: Arc<S>,
}

impl<S: BaseStore> TrieView<S> {
    pub(crate) fn load(store: S) -> Self {
        Self {
            past: None,
            store: Arc::new(store),
        }
    }
}

impl<S: BaseStore> Clone for TrieView<S> {
    fn clone(&self) -> Self {
        let past = self.past.clone();
        let store = self.store.clone();
        Self { past, store }
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for TrieView<S> {
    type Subtrie = TrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl<S: BaseStore + Send + Sync> BaseView for TrieView<S> {
    type Snapshot = TrieView<S>;

    fn status(&self) -> StorageStatus {
        StorageStatus {
            max_id: self.store.max_id(),
            root: self.store.root(),
        }
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let store = Arc::new(self.store.with_root(new_root));
        Self { store, ..self }
    }
    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl<S: BaseStore> BaseRead for TrieView<S> {
    fn read_root(&self) -> MapBase {
        self.store.root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id >= self.store.start_id() {
            self.store.base(id).await
        } else if let Some(past) = &self.past {
            Box::pin(past.read_base(id)).await
        } else {
            Ok(Base::empty())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::{Mem, TrieView};
    use crate::trie::TrieStream;
    use futures::StreamExt;

    #[tokio::test]
    async fn stream_exists() {
        let trie = TrieView::load(Mem::new());
        let stream = trie.u32_stream();
        let values: Vec<(i32, u32)> = stream.collect().await;
        assert_eq!(values, vec![]);
    }
}
