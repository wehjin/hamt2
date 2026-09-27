use crate::storage::load::StoreLoad;
use crate::storage::trie_edit::TrieEdit;
use crate::storage::{MemLoad, TrieView};
use crate::trie::{MapBase, TrieQuery, TrieQueryError, TrieSnap, TrieStream, TrieValue, TrieWalk};
use anyhow::Error;
use futures::Stream;

pub fn mem_load_new() -> TrieLoad<MemLoad> {
    TrieLoad::new()
}

#[derive(Debug)]
pub struct TrieLoad<S: StoreLoad> {
    inner: S,
}

impl<S: StoreLoad> StoreLoad for TrieLoad<S> {
    type Edit = TrieEdit<S::Edit>;
    type View = TrieView<S::View>;

    fn new() -> Self {
        Self { inner: S::new() }
    }

    async fn begin_edit(&self) -> Result<Self::Edit, Error> {
        let edit = TrieEdit {
            inner: self.inner.begin_edit().await?,
        };
        Ok(edit)
    }

    async fn commit_edit(&mut self, edit: Self::Edit) -> Result<(), Error> {
        let edit = edit.inner;
        self.inner.commit_edit(edit).await
    }
}

impl<S: StoreLoad> TrieWalk for TrieLoad<S> {
    type Subtrie = TrieView<S::View>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        TrieView {
            inner: self.inner.to_subtrie(subtrie_root),
        }
    }
}

impl<S: StoreLoad> TrieStream for TrieLoad<S> {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        self.inner.map_base_stream()
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.inner.u32_stream()
    }
}

impl<S: StoreLoad> TrieSnap for TrieLoad<S> {
    type Snapshot = TrieView<S::View>;

    fn snapshot(&self) -> Self::Snapshot {
        TrieView {
            inner: self.inner.snapshot(),
        }
    }
}

impl<S: StoreLoad> TrieQuery for TrieLoad<S> {
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
