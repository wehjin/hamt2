use crate::{MapBase, TrieValue};

pub trait Query {
    fn get_root(&self) -> MapBase;

    /// Returns the value stored at the given key or none if the key is absent.
    fn query(&self, key: i32) -> Option<TrieValue>;

    /// Returns u32 value
    fn query_u32(&self, key: i32) -> Option<u32> {
        if let Some(TrieValue::U32(value)) = self.query(key) {
            Some(value)
        } else {
            None
        }
    }

    /// Returns all keys and values in this trie.
    fn query_all(&self) -> Vec<(i32, TrieValue)>;

    /// Returns the value stored at the given deep key.
    fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue>;
}
