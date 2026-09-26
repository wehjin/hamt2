use crate::storage::MemEdit;
use crate::storage::load::StoreLoad;
use crate::storage::mem::MemView;
use crate::trie::{
    Base, BaseId, BaseRead, MapBase, TrieQuery, TrieQueryError, TrieSnap, TrieStream, TrieValue,
    TrieWalk,
};
use futures::Stream;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deliberately non-Clone
#[derive(Debug)]
pub struct MemLoad {
    pub(crate) inner: MemView,
}

impl MemLoad {}

impl StoreLoad for MemLoad {
    type Edit = MemEdit;

    fn new() -> Self {
        let inner = MemView {
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId(0),
            root: MapBase::empty(),
        };
        Self { inner }
    }

    async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut Self::Edit) -> anyhow::Result<Out>,
    {
        let mut edit = MemEdit {
            past: self.inner.snapshot(),
            bases: vec![],
            root: self.inner.read_root(),
        };
        let out = f(&mut edit).await?;
        {
            let max_id = edit.max_id();
            let root = edit.root;
            let bases = self.inner.bases.clone();
            bases.write().await.append(&mut edit.bases);
            self.inner = MemView {
                bases,
                max_id,
                root,
            };
        }
        Ok(out)
    }
}

impl TrieWalk for MemLoad {
    type Subtrie = MemView;
    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        MemView {
            root: subtrie_root,
            ..self.snapshot()
        }
    }
}

impl TrieStream for MemLoad {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        self.inner.map_base_stream()
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.inner.u32_stream()
    }
}

impl TrieSnap for MemLoad {
    type Snapshot = MemView;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl TrieQuery for MemLoad {
    async fn query(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        self.inner.query(key).await
    }

    async fn query_u32(&self, key: i32) -> Result<Option<u32>, TrieQueryError> {
        self.inner.query_u32(key).await
    }

    async fn query_all(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        self.inner.query_all().await
    }

    async fn deep_query<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        self.inner.deep_query(key).await
    }
}
