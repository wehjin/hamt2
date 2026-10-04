use crate::TrieValue;
use crate::services::map_base;
use crate::{Buffer, HashKey};

pub trait Query {
    /// Returns the value stored at the given key or none if the key is absent.
    fn query(&self, key: i32) -> Option<TrieValue>;

    fn query_u32(&self, key: i32) -> Option<u32>;

    /// Returns all keys and values in this trie.
    fn query_all(&self) -> Vec<(i32, TrieValue)>;

    /// Returns the value stored at the given deep key.
    fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue>;
}

impl<T: Buffer> Query for T {
    fn query(&self, key: i32) -> Option<TrieValue> {
        map_base::query_value(self.get_root(), HashKey::new(key), self)
    }

    fn query_u32(&self, key: i32) -> Option<u32> {
        if let Some(TrieValue::U32(value)) = self.query(key) {
            Some(value)
        } else {
            None
        }
    }

    fn query_all(&self) -> Vec<(i32, TrieValue)> {
        map_base::query_keys_values(self.get_root(), self)
    }

    fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue> {
        map_base::query_value_deep(self.get_root(), key, self)
    }
}
