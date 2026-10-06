use super::VecBuffer;
use crate::{Base, Trie};
use crate::{Buffer, BufferIndex, MapBase, QueryCursor, Snap};
use crate::{CursorPos, Slot};
use crate::{HashKey, Query, SkyTrieMut, TrieValue, map_base};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SkyTrie {
    pub(crate) buffer: Arc<VecBuffer>,
    pub(crate) cursor_pos: CursorPos,
}
impl Eq for SkyTrie {}

impl PartialEq for SkyTrie {
    fn eq(&self, other: &Self) -> bool {
        self.cursor_pos == other.cursor_pos
            && self.buffer == other.buffer
    }
}

/// Constructors
impl SkyTrie {
    pub fn new() -> Self {
        // All tries start with a single slot containing the root of the trie.
        let seed = Slot::MapBase(MapBase::empty());
        let buffer = VecBuffer::new(seed);
        Self::with_buffer(buffer)
    }

    pub fn with_buffer(buffer: VecBuffer) -> Self {
        let max_id = buffer.max_index();
        let root = buffer.get_subtrie(max_id);
        let cursor_pos = CursorPos::new(root);
        Self {
            buffer: Arc::new(buffer),
            cursor_pos,
        }
    }

    pub async fn into_edit(self) -> SkyTrieMut {
        SkyTrieMut::extend(self)
    }

    pub async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut SkyTrieMut) -> anyhow::Result<Out>,
    {
        let past = self.clone();
        let mut edit = past.into_edit().await;
        let out = f(&mut edit).await?;
        let next = edit.commit().await;
        *self = next;
        Ok(out)
    }
}

/// Queries
impl SkyTrie {
    pub fn list_keys(&self) -> Vec<i32> {
        self.clone().into_iter().map(|(key, _)| key).collect()
    }
}

impl QueryCursor for SkyTrie {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }

    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}

impl Query for SkyTrie {
    fn get_root(&self) -> MapBase {
        self.active_root()
    }

    fn query(&self, key: i32) -> Option<TrieValue> {
        map_base::query_value(self.get_root(), HashKey::new(key), self)
    }

    fn query_all(&self) -> Vec<(i32, TrieValue)> {
        map_base::query_keys_values(self.get_root(), self)
    }

    fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue> {
        map_base::query_value_deep(self.get_root(), key, self)
    }
}

impl Buffer for SkyTrie {
    fn max_index(&self) -> BufferIndex {
        self.buffer.max_index()
    }

    fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        self.buffer.get_base(id, size)
    }
}

impl Snap for SkyTrie {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl Trie for SkyTrie {}
