use crate::SkyTrie;
use crate::TrieMut;
use crate::{CursorPos, Insert, MapBase, Snap, TrieValue};
use crate::{HashKey, InsertOption, Query, map_base};
use crate::{InsertCursor, QueryCursor};
use std::collections::HashSet;

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct SkyTrieMut {
    pub(crate) cursor_pos: CursorPos,
}

impl SkyTrieMut {
    pub fn new() -> Self {
        Self::extend(SkyTrie::new())
    }

    pub fn extend(past: SkyTrie) -> Self {
        let cursor_pos = past.cursor_pos().clone().ascend_top();
        Self { cursor_pos }
    }
    pub fn commit(self) -> SkyTrie {
        let cursor_pos = self.cursor_pos.ascend_top();
        SkyTrie { cursor_pos }
    }

    pub(crate) fn push_root(&mut self, root: MapBase) {
        if let Some((key, _previous_active)) = self.cursor_pos.ascend() {
            // Make sure the ascended level has the updated value at key.
            let subtrie = TrieValue::SubTrie(root.clone());
            self.insert(key.into(), subtrie);
            // Return to the original level.
            self.cursor_pos.descend(key, root);
        } else {
            self.cursor_pos.active_root = root;
        }
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
    fn query(&self, key: i32) -> Option<TrieValue> {
        map_base::query_value(self.active_root(), HashKey::new(key))
    }

    fn query_all(&self) -> Vec<(i32, TrieValue)> {
        map_base::query_keys_values(self.active_root())
    }

    fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue> {
        map_base::query_value_deep(self.active_root(), key)
    }
}
impl InsertCursor for SkyTrieMut {}

impl Insert for SkyTrieMut {
    fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> &mut Self {
        let options = options.into_iter().collect::<HashSet<_>>();
        let pre_root = if options.contains(&InsertOption::DeleteOthers) {
            MapBase::empty()
        } else {
            self.active_root()
        };
        let value = value.into();
        let key = HashKey::new(key);
        let root = map_base::insert_kv(pre_root, key, value);
        self.push_root(root);
        self
    }
}

impl Snap for SkyTrieMut {
    type Snapshot = SkyTrie;

    fn snapshot(&self) -> Self::Snapshot {
        SkyTrie {
            cursor_pos: self.cursor_pos.clone(),
        }
    }
}
