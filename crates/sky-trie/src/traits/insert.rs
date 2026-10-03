use crate::services::map_base;
use crate::{BufferMut, HashKey, MapBase, TrieValue};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InsertOption {
    /// Clear the trie of other values before inserting.
    DeleteOthers,
}

#[allow(async_fn_in_trait)]
pub trait Insert {
    /// Inserts `value` into the trie at position `key`.
    async fn insert(&mut self, key: i32, value: impl Into<TrieValue>) -> &mut Self {
        self.insert_with_options(key, value, []).await
    }

    /// Inserts `value` into the trie at position `key` with `options`.
    async fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> &mut Self;
}

impl<T: BufferMut> Insert for T {
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
