use crate::storage::ReadStorageError;
use crate::storage::traits::view::StoreView;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieSnap, TrieWalk};
use std::sync::Arc;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests {
    use crate::storage::MemView;
    use crate::trie::{TrieQuery, TrieSnap, TrieStream, TrieWalk};
    use futures::StreamExt;

    #[tokio::test]
    async fn has_query() {
        let mem = MemView::new();
        let values = mem.query_all().await.unwrap();
        assert_eq!(values, vec![]);
    }

    #[tokio::test]
    async fn has_snap() {
        let mem = MemView::new();
        let values = mem.snapshot().query_all().await.unwrap();
        assert_eq!(values, vec![]);
    }

    #[tokio::test]
    async fn has_stream() {
        let mem = MemView::new();
        let values = mem.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(values, vec![]);
    }

    #[tokio::test]
    async fn has_walk() {
        let mem = MemView::new();
        let subtries = mem.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(subtries.len(), 0);
    }
}

#[derive(Debug, Clone)]
pub struct MemView {
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) max_id: BaseId,
    pub(crate) root: MapBase,
}

impl MemView {
    pub fn new() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId(0),
            root: MapBase::empty(),
        }
    }
}

impl BaseRead for MemView {
    fn max_id(&self) -> BaseId {
        self.max_id
    }

    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let base = if id < BaseId::ZERO || id > self.max_id {
            Base::empty()
        } else {
            let index = id.0 as usize;
            self.bases.read().await[index].clone()
        };
        Ok(base)
    }
}

impl TrieSnap for MemView {
    type Snapshot = Self;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let root = new_root.unwrap_or(self.root);
        Self { root, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl TrieWalk for MemView {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl StoreView for MemView {}
