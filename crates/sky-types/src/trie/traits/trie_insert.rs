use crate::trie::{BufferMut, HashKey, MapBase, TrieInsertError, TrieValue, map_base};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InsertOption {
    /// Clear the trie of other values before inserting.
    DeleteOthers,
}

#[allow(async_fn_in_trait)]
pub trait TrieInsert {
    /// Inserts `value` into the trie at position `key`.
    async fn insert(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
    ) -> Result<&mut Self, TrieInsertError> {
        self.insert_with_options(key, value, []).await
    }

    /// Inserts `value` into the trie at position `key` with `options`.
    async fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> Result<&mut Self, TrieInsertError>;
}

impl<T: BufferMut> TrieInsert for T {
    async fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> Result<&mut Self, TrieInsertError> {
        let options = options.into_iter().collect::<HashSet<_>>();
        let pre_root = if options.contains(&InsertOption::DeleteOthers) {
            MapBase::empty()
        } else {
            self.read_root()
        };
        let value = value.into();
        let key = HashKey::new(key);
        let root = map_base::insert_kv(pre_root, key, value, self).await?;
        self.commit_root(root).await?;
        Ok(self)
    }
}
