use crate::{MapBase, TrieValue};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InsertOption {
    /// Clear the trie of other values before inserting.
    DeleteOthers,
}

#[allow(async_fn_in_trait)]
pub trait Insert {
    /// Commits a new `root` into the trie.
    async fn push_root(&mut self, root: MapBase);

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
