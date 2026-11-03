use crate::TrieValue;

pub trait Insert {
    /// Inserts `value` into the trie at position `key`.
    fn insert(&mut self, key: i32, value: impl Into<TrieValue>) -> &mut Self;
}
