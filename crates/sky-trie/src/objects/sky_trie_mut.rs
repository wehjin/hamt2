use super::VecBuffer;
use crate::{Base, Buffer, BufferIndex, BufferMut, CursorPos, Insert, MapBase, Snap, TrieValue};
use crate::{HashKey, Query, SkyTrie, map_base};
use crate::{InsertCursor, QueryCursor};
use crate::{InsertOption, TrieMut};
use std::collections::HashSet;
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

    pub fn extend(past: SkyTrie) -> Self {
        let cursor_pos = past.cursor_pos().clone().ascend_top();
        let buffer = VecBuffer::extend(past.buffer.clone());
        let past = Arc::new(past);
        Self {
            past,
            buffer,
            cursor_pos,
        }
    }
    pub async fn commit(self) -> SkyTrie {
        let buffer = self.buffer.retract();
        SkyTrie::with_buffer(buffer)
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

impl Query for SkyTrieMut {
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

impl Buffer for SkyTrieMut {
    fn max_index(&self) -> BufferIndex {
        self.buffer.max_index()
    }

    fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        self.buffer.get_base(id, size)
    }
}
impl InsertCursor for SkyTrieMut {}

impl Insert for SkyTrieMut {
    async fn push_root(&mut self, root: MapBase) {
        if let Some((key, _previous_active)) = self.cursor_pos.ascend() {
            // Make sure the ascended level has the updated value at key.
            let subtrie = TrieValue::SubTrie(root);
            Box::pin(self.insert(key.into(), subtrie)).await;
            // Return to the original level.
            self.cursor_pos.descend(key, root);
        } else {
            self.buffer.push_subtrie(root).await;
            self.cursor_pos.active_root = root;
        }
    }
    async fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> &mut Self {
        let options = options.into_iter().collect::<HashSet<_>>();
        let pre_root = if options.contains(&InsertOption::DeleteOthers) {
            MapBase::empty()
        } else {
            self.get_root()
        };
        let value = value.into();
        let key = HashKey::new(key);
        let root = map_base::insert_kv(pre_root, key, value, self).await;
        self.push_root(root).await;
        self
    }
}

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

impl BufferMut for SkyTrieMut {
    async fn push_base(&mut self, base: Base) -> BufferIndex {
        self.buffer.push_base(base).await
    }
}
