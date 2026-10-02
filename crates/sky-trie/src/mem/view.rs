use crate::mem::SlotBuffer;
use crate::CursorPos;
use crate::MemEdit;
use crate::StoreView;
use crate::{Base, Buffer, BufferIndex, MapBase, QueryCursor, TrieSnap};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MemView {
    pub(crate) buffer: Arc<SlotBuffer>,
    pub(crate) max_id: BufferIndex,
    pub(crate) cursor_pos: CursorPos,
}

impl MemView {
    pub fn new() -> Self {
        Self::with_buffer(SlotBuffer::new())
    }

    pub fn with_buffer(buffer: SlotBuffer) -> Self {
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

    pub async fn into_edit(self) -> MemEdit {
        MemEdit::extend(self)
    }

    pub async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut MemEdit) -> anyhow::Result<Out>,
    {
        let past = self.clone();
        let mut edit = past.into_edit().await;
        let out = f(&mut edit).await?;
        let next = edit.commit().await;
        *self = next;
        Ok(out)
    }
}

impl QueryCursor for MemView {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }

    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}

impl Eq for MemView {}

impl PartialEq for MemView {
    fn eq(&self, other: &Self) -> bool {
        self.cursor_pos == other.cursor_pos
            && self.max_id == other.max_id
            && self.buffer == other.buffer
    }
}

impl Buffer for MemView {
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

impl TrieSnap for MemView {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl StoreView for MemView {}
