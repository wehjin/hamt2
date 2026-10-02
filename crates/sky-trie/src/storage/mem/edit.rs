use crate::storage::edit::StoreEdit;
use crate::storage::{MemView, SlotBuffer, WriteStorageError};
use crate::trie::{
    Base, Buffer, BufferIndex, BufferMut, CursorPos, MapBase, TrieInsert, TrieSnap, TrieValue,
};
use crate::trie::{InsertCursor, QueryCursor};
use std::ops::Deref;
use std::sync::Arc;

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct MemEdit {
    pub(crate) past: Arc<MemView>,
    pub(crate) buffer: SlotBuffer,
    pub(crate) cursor_pos: CursorPos,
}

impl MemEdit {
    pub fn new() -> Self {
        Self::extend(MemView::new())
    }

    pub async fn commit(self) -> MemView {
        let mut past_buffer = self.past.buffer.deref().clone();
        past_buffer.append(self.buffer);
        MemView::with_buffer(past_buffer)
    }

    pub fn extend(past: MemView) -> Self {
        let past = Arc::new(past);
        let buffer = SlotBuffer::new();
        let cursor_pos = past.cursor_pos().clone().ascend_top();
        Self {
            past,
            buffer,
            cursor_pos,
        }
    }

    fn start_id(&self) -> BufferIndex {
        self.past.max_index() + 1
    }
}

impl QueryCursor for MemEdit {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }
    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}
impl InsertCursor for MemEdit {}
impl StoreEdit for MemEdit {}

impl TrieSnap for MemEdit {
    type Snapshot = MemEdit;

    fn snapshot(&self) -> Self::Snapshot {
        Self {
            past: self.past.clone(),
            buffer: self.buffer.clone(),
            cursor_pos: self.cursor_pos.clone(),
        }
    }
}
impl Buffer for MemEdit {
    fn max_index(&self) -> BufferIndex {
        self.past.max_index() + self.buffer.len()
    }

    fn get_root(&self) -> MapBase {
        self.cursor_pos.active_root()
    }

    async fn get_base(&self, id: BufferIndex, slots: usize) -> Base {
        let start_id = self.start_id();
        if id < start_id {
            self.past.get_base(id, slots).await
        } else if id <= self.max_index() {
            let buffer_index = BufferIndex(id.0 - start_id.0);
            self.buffer.get_base(buffer_index, slots).await
        } else {
            Base::empty()
        }
    }
}

impl BufferMut for MemEdit {
    async fn push_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        let cursor_pos = self.cursor_pos.clone();
        if let Some((key, _previous_active)) = self.cursor_pos.ascend() {
            // Make sure the ascended level has the updated value at key.
            let subtrie = TrieValue::SubTrie(root);
            if let Err(e) = Box::pin(self.insert(key.into(), subtrie)).await {
                self.cursor_pos = cursor_pos;
                return Err(WriteStorageError::Anyhow(e.into()));
            }
            // Return to the original level.
            self.cursor_pos.descend(key, root);
        } else {
            self.buffer.push_root(root).await?;
            self.cursor_pos.active_root = root;
        }
        Ok(())
    }

    async fn push_base(&mut self, base: Base) -> Result<BufferIndex, WriteStorageError> {
        let base_size = base.len();
        let buffer_index = self.buffer.push_base(base).await?;
        let edit_index = self.start_id() + buffer_index.0;
        let next_index = self.next_index();
        debug_assert_eq!(next_index, edit_index + base_size);
        Ok(edit_index)
    }
}
