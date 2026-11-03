use crate::Trie;
use crate::{CursorPos, QueryCursor, Snap};
use crate::{HashKey, MapBase, Query, SkyTrieMut, TrieValue, map_base};

#[derive(Debug, Clone)]
pub struct SkyTrie {
    pub(crate) cursor_pos: CursorPos,
}

impl Eq for SkyTrie {}

impl PartialEq for SkyTrie {
    fn eq(&self, other: &Self) -> bool {
        self.cursor_pos == other.cursor_pos
    }
}

/// Constructors
impl SkyTrie {
    pub fn new() -> Self {
        let cursor_pos = CursorPos::new(MapBase::empty());
        Self { cursor_pos }
    }

    pub fn into_edit(self) -> SkyTrieMut {
        SkyTrieMut::extend(self)
    }

    pub fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: FnOnce(&mut SkyTrieMut) -> anyhow::Result<Out>,
    {
        let past = self.clone();
        let mut edit = past.into_edit();
        let out = f(&mut edit)?;
        let next = edit.commit();
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

/// Persistent inserts
impl SkyTrie {
    /// Inserts `value` at `key`, returning a new trie. The original is unchanged.
    pub fn insert(&self, key: i32, value: impl Into<TrieValue>) -> SkyTrie {
        let root = self.top_root();
        let new_root = map_base::insert_kv(root, HashKey::new(key), value.into());
        SkyTrie {
            cursor_pos: CursorPos::new(new_root),
        }
    }

    /// Inserts `value` at the deep path `key`, returning a new trie. The original is unchanged.
    pub fn insert_deep<const N: usize>(&self, key: [i32; N], value: impl Into<TrieValue>) -> SkyTrie {
        let root = self.top_root();
        let new_root = map_base::insert_kv_deep(root, key, value.into());
        SkyTrie {
            cursor_pos: CursorPos::new(new_root),
        }
    }

    /// Returns the subtrie at the deep path `key`, if it exists.
    pub fn subtrie<const N: usize>(&self, key: [i32; N]) -> Option<MapBase> {
        match map_base::query_value_deep(self.top_root(), key) {
            Some(TrieValue::SubTrie(mb)) => Some(mb),
            _ => None,
        }
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

impl Snap for SkyTrie {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl Trie for SkyTrie {}

#[cfg(test)]
mod tests {
    use crate::{HashKey, Query, SkyTrie, TrieValue, map_base};

    #[test]
    fn insert_returns_new_trie() {
        let trie = SkyTrie::new();
        let next = trie.insert(7, TrieValue::U32(7));
        assert_eq!(trie.query(7), None);
        assert_eq!(next.query(7), Some(TrieValue::U32(7)));
    }

    #[test]
    fn insert_deep_returns_new_trie() {
        let trie = SkyTrie::new();
        let next = trie.insert_deep([1, 2, 3], TrieValue::U32(42));
        assert_eq!(trie.query_deep([1, 2, 3]), None);
        assert_eq!(next.query_deep([1, 2, 3]), Some(TrieValue::U32(42)));
    }

    #[test]
    fn subtrie_returns_subtrie() {
        let trie = SkyTrie::new().insert_deep([1, 2, 3], TrieValue::U32(42));
        let sub = trie.subtrie([1, 2]).expect("subtrie should exist");
        assert_eq!(
            map_base::query_value(sub, HashKey::new(3)),
            Some(TrieValue::U32(42))
        );
        assert_eq!(trie.subtrie([1, 9]), None);
    }
}
