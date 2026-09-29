use crate::storage::ReadStorageError;
use crate::storage::traits::view::StoreView;
use crate::trie::{Base, BaseId, BaseRead, MapBase, QueryCursor, TrieSnap};
use crate::trie::{CursorPos, TrieWalk};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MemView {
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) max_id: BaseId,
    pub(crate) cursor_pos: CursorPos,
}

impl MemView {
    pub fn new() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId::ZERO,
            cursor_pos: CursorPos::new(MapBase::empty()),
        }
    }

    pub fn top_root(&self) -> MapBase {
        self.read_root()
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
            && self.bases.deref().read().unwrap().deref()
                == other.bases.deref().read().unwrap().deref()
    }
}

impl BaseRead for MemView {
    fn max_id(&self) -> BaseId {
        self.max_id
    }

    fn read_root(&self) -> MapBase {
        self.cursor_pos.active_root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let base = if id < BaseId::ZERO || id > self.max_id {
            Base::empty()
        } else {
            let index = id.0 as usize;
            self.bases.read().unwrap()[index].clone()
        };
        Ok(base)
    }
}

impl TrieSnap for MemView {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl TrieWalk for MemView {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        let cursor_pos = CursorPos::new(subtrie_root);
        MemView {
            cursor_pos,
            ..self.clone()
        }
    }
}

impl StoreView for MemView {}
