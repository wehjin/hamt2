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

/// Queries
impl SkyTrie {
    pub fn list_keys(&self) -> Vec<i32> {
        self.clone().into_iter().map(|(key, _)| key).collect()
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
