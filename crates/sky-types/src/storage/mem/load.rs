use crate::storage::MemEdit;
use crate::storage::load::StoreLoad;
use crate::storage::mem::MemView;
use crate::trie::{
    Buffer, CursorPos, MapBase, QueryCursor, TrieQuery, TrieQueryError, TrieSnap, TrieStream,
    TrieValue,
};
use anyhow::anyhow;
use futures::Stream;
use std::ops::Deref;

/// Deliberately non-Clone
#[derive(Debug)]
pub struct MemLoad {
    pub(crate) inner: MemView,
}

impl MemLoad {}

impl QueryCursor for MemLoad {
    fn cursor_pos(&self) -> &CursorPos {
        self.inner.cursor_pos()
    }

    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        self.inner.cursor_pos_mut()
    }
}

impl StoreLoad for MemLoad {
    type Edit = MemEdit;
    type View = MemView;

    fn new() -> Self {
        let inner = MemView::new();
        Self { inner }
    }

    async fn begin_edit(&self) -> Result<Self::Edit, anyhow::Error> {
        let edit = MemEdit::extend(self.inner.snapshot());
        Ok(edit)
    }
    async fn commit_edit(&mut self, edit: Self::Edit) -> Result<(), anyhow::Error> {
        if edit.past.max_index() != self.inner.max_index()
            || edit.past.get_root() != self.inner.get_root()
        {
            return Err(anyhow!("stale edit"));
        }
        let mut buffer = self.inner.buffer.deref().clone();
        buffer.append(edit.buffer);
        self.inner = MemView::with_buffer(buffer);
        Ok(())
    }
}

impl TrieStream for MemLoad {
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        self.inner.map_base_stream()
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.inner.u32_stream()
    }

    fn kv_stream(&self) -> impl Stream<Item = (i32, TrieValue)> {
        self.inner.kv_stream()
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

    async fn query_deep<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        self.inner.query_deep(key).await
    }
}
