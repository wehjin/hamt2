use crate::storage::MemEdit;
use crate::storage::load::StoreLoad;
use crate::storage::mem::MemView;
use crate::trie::{
    Base, BaseId, BaseRead, MapBase, TrieQuery, TrieQueryError, TrieSnap, TrieStream, TrieValue,
    TrieWalk,
};
use anyhow::anyhow;
use futures::Stream;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

/// Deliberately non-Clone
#[derive(Debug)]
pub struct MemLoad {
    pub(crate) inner: MemView,
}

impl MemLoad {}

impl StoreLoad for MemLoad {
    type Edit = MemEdit;
    type View = MemView;

    fn new() -> Self {
        let inner = MemView {
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId(0),
            root: MapBase::empty(),
        };
        Self { inner }
    }

    async fn begin_edit(&self) -> Result<Self::Edit, anyhow::Error> {
        let edit = MemEdit::extend(self.inner.snapshot());
        Ok(edit)
    }
    async fn commit_edit(&mut self, edit: Self::Edit) -> Result<(), anyhow::Error> {
        if edit.past.max_id() != self.inner.max_id()
            || edit.past.read_root() != self.inner.read_root()
        {
            return Err(anyhow!("stale edit"));
        }
        let max_id = edit.max_id();
        let root = edit.top_root();
        let bases = {
            let mut edit_bases = edit
                .bases
                .into_iter()
                .map(|base| base.deref().clone())
                .collect();
            let bases = self.inner.bases.clone();
            bases.write().unwrap().append(&mut edit_bases);
            bases
        };
        self.inner = MemView {
            bases,
            max_id,
            root,
        };
        Ok(())
    }
}

impl TrieWalk for MemLoad {
    type Subtrie = <Self as StoreLoad>::View;
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
    type Snapshot = <Self as StoreLoad>::View;

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
