use crate::TrieValue;
use crate::services::map_base;
use crate::{Buffer, HashKey};

#[allow(async_fn_in_trait)]
pub trait Query {
    /// Returns the value stored at the given key or none if the key is absent.
    async fn query(&self, key: i32) -> Option<TrieValue>;

    async fn query_u32(&self, key: i32) -> Option<u32>;

    /// Returns all keys and values in this trie.
    async fn query_all(&self) -> Vec<(i32, TrieValue)>;

    /// Returns the value stored at the given deep key.
    async fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue>;
}

impl<T: Buffer> Query for T {
    async fn query(&self, key: i32) -> Option<TrieValue> {
        map_base::query_value(self.get_root(), HashKey::new(key), self).await
    }

    async fn query_u32(&self, key: i32) -> Option<u32> {
        if let Some(TrieValue::U32(value)) = self.query(key).await {
            Some(value)
        } else {
            None
        }
    }

    async fn query_all(&self) -> Vec<(i32, TrieValue)> {
        map_base::query_keys_values(self.get_root(), self).await
    }

    async fn query_deep<const N: usize>(&self, key: [i32; N]) -> Option<TrieValue> {
        map_base::query_value_deep(self.get_root(), key, self).await
    }
}
