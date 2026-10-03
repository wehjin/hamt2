use crate::CursorPos;
use crate::Trie;
use crate::SkyTrieMut;
use crate::objects::VecBuffer;
use crate::{Base, Buffer, BufferIndex, MapBase, QueryCursor, Snap};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SkyTrie {
    pub(crate) buffer: Arc<VecBuffer>,
    pub(crate) max_id: BufferIndex,
    pub(crate) cursor_pos: CursorPos,
}

impl SkyTrie {
    pub fn new() -> Self {
        Self::with_buffer(VecBuffer::new())
    }

    pub fn with_buffer(buffer: VecBuffer) -> Self {
        let max_id = buffer.max_index();
        let cursor_pos = CursorPos::new(buffer.get_root());
        Self {
            buffer: Arc::new(buffer),
            max_id,
            cursor_pos,
        }
    }

    pub fn top_root(&self) -> MapBase {
        self.get_root()
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

impl QueryCursor for SkyTrie {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }

    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}

impl Eq for SkyTrie {}

impl PartialEq for SkyTrie {
    fn eq(&self, other: &Self) -> bool {
        self.cursor_pos == other.cursor_pos
            && self.max_id == other.max_id
            && self.buffer == other.buffer
    }
}

impl Buffer for SkyTrie {
    fn max_index(&self) -> BufferIndex {
        self.max_id
    }

    fn get_root(&self) -> MapBase {
        self.cursor_pos.active_root
    }

    async fn get_base(&self, id: BufferIndex, slots: usize) -> Base {
        if id < BufferIndex::ZERO || id > self.max_id {
            Base::empty()
        } else {
            self.buffer.get_base(id, slots).await
        }
    }
}

impl Snap for SkyTrie {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl Trie for SkyTrie {}
