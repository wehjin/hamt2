use crate::storage::load::StoreLoad;
use crate::storage::mem::MemView;
use crate::trie::{MapBase, TrieQuery, TrieQueryError, TrieSnap, TrieStream, TrieValue, TrieWalk};
use futures::Stream;

/// Deliberately non-Clone
#[derive(Debug)]
pub struct MemLoad {
    pub(crate) inner: MemView,
}

impl StoreLoad for MemLoad {}

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
