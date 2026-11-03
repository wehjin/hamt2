use crate::TrieValue;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum InsertOption {
    /// Clear the trie of other values before inserting.
    DeleteOthers,
}

pub trait Insert {
    /// Inserts `value` into the trie at position `key`.
    fn insert(&mut self, key: i32, value: impl Into<TrieValue>) -> &mut Self {
        self.insert_with_options(key, value, [])
    }

    /// Inserts `value` into the trie at position `key` with `options`.
    fn insert_with_options(
        &mut self,
        key: i32,
        value: impl Into<TrieValue>,
        options: impl IntoIterator<Item = InsertOption>,
    ) -> &mut Self;
}
