use crate::storage::edit::StoreEdit;
use crate::storage::{MemView, ReadStorageError, WriteStorageError};
use crate::trie::{
    Base, BaseCommit, BaseId, BaseRead, CursorPos, MapBase, TrieInsert, TrieSnap, TrieValue,
};
use crate::trie::{InsertCursor, QueryCursor};
use std::ops::Deref;
use std::sync::Arc;

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct MemEdit {
    pub(crate) past: Arc<MemView>,
    pub(crate) bases: Vec<Arc<Base>>,
    pub(crate) cursor_pos: CursorPos,
}

impl MemEdit {
    pub fn new() -> Self {
        Self::extend(MemView::new())
    }
    pub fn extend(past: MemView) -> Self {
        let past = Arc::new(past);
        let cursor_pos = past.cursor_pos().clone().ascend_top();
        let bases = vec![];
        Self {
            past,
            bases,
            cursor_pos,
        }
    }

    fn start_id(&self) -> BaseId {
        self.past.max_id() + 1
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
            bases: self.bases.clone(),
            cursor_pos: self.cursor_pos.clone(),
        }
    }
}
impl BaseCommit for MemEdit {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        let cursor_pos = self.cursor_pos.clone();
        if let Some((key, _previous_active)) = self.cursor_pos.ascend() {
            // Make sure the ascended level has the updated value at key.
            let value = TrieValue::SubTrie(root);
            if let Err(e) = Box::pin(self.insert(key.into(), value)).await {
                self.cursor_pos = cursor_pos;
                return Err(WriteStorageError::Anyhow(e.into()));
            }
            // Return to the original level.
            self.cursor_pos.descend(key, root);
        } else {
            self.cursor_pos.active_root = root;
        }
        Ok(())
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let next_id = self.next_id();
        self.bases.push(Arc::new(base));
        debug_assert_eq!(self.max_id(), next_id);
        Ok(next_id)
    }
}

impl BaseRead for MemEdit {
    fn max_id(&self) -> BaseId {
        self.past.max_id() + self.bases.len()
    }

    fn read_root(&self) -> MapBase {
        self.cursor_pos.active_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let start_id = self.start_id();
        if id < start_id {
            return self.past.read_base(id).await;
        }
        if id <= self.max_id() {
            let index = (id.0 - start_id.0) as usize;
            let base = self.bases[index].deref().clone();
            Ok(base)
        } else {
            Ok(Base::empty())
        }
    }
}
