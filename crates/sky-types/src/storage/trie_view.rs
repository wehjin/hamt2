use crate::storage::view::StoreView;
use crate::trie::TrieWalk;
use crate::trie::{MapBase, TrieQuery, TrieQueryError, TrieValue};
use crate::trie::{TrieSnap, TrieStream};
use futures::Stream;

#[derive(Debug, Clone)]
pub struct TrieView<S: StoreView> {
    pub(crate) inner: S,
}

impl<S: StoreView> TrieWalk for TrieView<S> {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        Self {
            inner: self.inner.to_subtrie(subtrie_root),
        }
    }
}

impl<S: StoreView> StoreView for TrieView<S> {}

impl<S: StoreView> TrieStream for TrieView<S> {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        self.inner.map_base_stream()
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.inner.u32_stream()
    }
}

impl<S: StoreView> TrieSnap for TrieView<S> {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        Self {
            inner: self.inner.snapshot(),
        }
    }
}

impl<S: StoreView> TrieQuery for TrieView<S> {
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

#[cfg(test)]
mod tests {
    use crate::storage::load::StoreLoad;
    use crate::storage::{MemLoad, TrieView};
    use crate::trie::{TrieSnap, TrieStream};
    use futures::StreamExt;

    #[tokio::test]
    async fn stream_exists() {
        let trie = TrieView {
            inner: MemLoad::new().snapshot(),
        };
        let values: Vec<(i32, u32)> = trie.u32_stream().collect().await;
        assert_eq!(values, vec![]);
    }
}
