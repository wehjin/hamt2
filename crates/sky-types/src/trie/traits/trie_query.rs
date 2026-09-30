use crate::trie::TrieQueryError;
use crate::trie::TrieValue;
use crate::trie::{Buffer, HashKey, map_base};

#[allow(async_fn_in_trait)]
pub trait TrieQuery {
    /// Returns the value stored at the given key or none if the key is absent.
    async fn query(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError>;

    async fn query_u32(&self, key: i32) -> Result<Option<u32>, TrieQueryError>;

    /// Returns all keys and values in this trie.
    async fn query_all(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError>;

    /// Returns the value stored at the given deep key.
    async fn query_deep<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError>;
}

impl<T: Buffer> TrieQuery for T {
    async fn query(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::query_value(self.get_root(), HashKey::new(key), self).await
    }

    async fn query_u32(&self, key: i32) -> Result<Option<u32>, TrieQueryError> {
        self.query(key).await.map(|value| {
            if let Some(TrieValue::U32(value)) = value {
                Some(value)
            } else {
                None
            }
        })
    }

    async fn query_all(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        map_base::query_keys_values(self.get_root(), self).await
    }

    async fn query_deep<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::query_value_deep(self.get_root(), key, self).await
    }
}
