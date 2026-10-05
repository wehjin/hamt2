use super::VecBuffer;
use crate::SkyTrie;
use crate::TrieMut;
use crate::{Base, Buffer, BufferIndex, BufferMut, CursorPos, Insert, MapBase, Snap, TrieValue};
use crate::{InsertCursor, QueryCursor};
use std::ops::Deref;
use std::sync::Arc;

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct SkyTrieMut {
    pub(crate) past: Arc<SkyTrie>,
    pub(crate) buffer: VecBuffer,
    pub(crate) cursor_pos: CursorPos,
}

impl SkyTrieMut {
    pub fn new() -> Self {
        Self::extend(SkyTrie::new())
    }

    pub async fn commit(self) -> SkyTrie {
        let mut past_buffer = self.past.buffer.deref().clone();
        past_buffer.append(self.buffer);
        SkyTrie::with_buffer(past_buffer)
    }

    pub fn extend(past: SkyTrie) -> Self {
        let past = Arc::new(past);
        let buffer = VecBuffer::new(None);
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

impl TrieMut for SkyTrieMut {}

impl QueryCursor for SkyTrieMut {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }
    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}
impl InsertCursor for SkyTrieMut {}

impl Snap for SkyTrieMut {
    type Snapshot = SkyTrieMut;

    fn snapshot(&self) -> Self::Snapshot {
        Self {
            past: self.past.clone(),
            buffer: self.buffer.clone(),
            cursor_pos: self.cursor_pos.clone(),
        }
    }
}
impl Buffer for SkyTrieMut {
    fn max_index(&self) -> BufferIndex {
        self.past.max_index() + self.buffer.len()
    }

    fn get_root(&self) -> MapBase {
        self.cursor_pos.active_root()
    }

    fn get_base(&self, id: BufferIndex, slots: usize) -> Base {
        let start_id = self.start_id();
        if id < start_id {
            self.past.get_base(id, slots)
        } else if id <= self.max_index() {
            let buffer_index = BufferIndex(id.0 - start_id.0);
            self.buffer.get_base(buffer_index, slots)
        } else {
            Base::empty()
        }
    }
}

impl BufferMut for SkyTrieMut {
    async fn push_root(&mut self, root: MapBase) {
        if let Some((key, _previous_active)) = self.cursor_pos.ascend() {
            // Make sure the ascended level has the updated value at key.
            let subtrie = TrieValue::SubTrie(root);
            Box::pin(self.insert(key.into(), subtrie)).await;
            // Return to the original level.
            self.cursor_pos.descend(key, root);
        } else {
            self.buffer.push_root(root).await;
            self.cursor_pos.active_root = root;
        }
    }

    async fn push_base(&mut self, base: Base) -> BufferIndex {
        let base_size = base.len();
        let buffer_index = self.buffer.push_base(base).await;
        let edit_index = self.start_id() + buffer_index.0;
        let next_index = self.next_index();
        debug_assert_eq!(next_index, edit_index + base_size);
        edit_index
    }
}
