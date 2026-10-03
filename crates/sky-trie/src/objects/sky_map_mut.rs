use crate::MapMut;
use crate::{Base, Buffer, BufferIndex, BufferMut, CursorPos, Insert, MapBase, Snap, TrieValue};
use crate::{InsertCursor, QueryCursor};
use crate::{SkyMap, VecBuffer};
use std::ops::Deref;
use std::sync::Arc;

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct SkyMapMut {
    pub(crate) past: Arc<SkyMap>,
    pub(crate) buffer: VecBuffer,
    pub(crate) cursor_pos: CursorPos,
}

impl SkyMapMut {
    pub fn new() -> Self {
        Self::extend(SkyMap::new())
    }

    pub async fn commit(self) -> SkyMap {
        let mut past_buffer = self.past.buffer.deref().clone();
        past_buffer.append(self.buffer);
        SkyMap::with_buffer(past_buffer)
    }

    pub fn extend(past: SkyMap) -> Self {
        let past = Arc::new(past);
        let buffer = VecBuffer::new();
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

impl MapMut for SkyMapMut {}

impl QueryCursor for SkyMapMut {
    fn cursor_pos(&self) -> &CursorPos {
        &self.cursor_pos
    }
    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        &mut self.cursor_pos
    }
}
impl InsertCursor for SkyMapMut {}

impl Snap for SkyMapMut {
    type Snapshot = SkyMapMut;

    fn snapshot(&self) -> Self::Snapshot {
        Self {
            past: self.past.clone(),
            buffer: self.buffer.clone(),
            cursor_pos: self.cursor_pos.clone(),
        }
    }
}
impl Buffer for SkyMapMut {
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

impl BufferMut for SkyMapMut {
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
