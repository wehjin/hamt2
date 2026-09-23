use crate::trie::TrieQueryError;
use crate::trie::TrieValue;
use crate::trie::{BaseRead, HashKey, map_base};

#[allow(async_fn_in_trait)]
pub trait TrieQuery: BaseRead {
    /// Returns the value stored at the given key or none if the key is absent.
    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError>;

    /// Returns all keys and values in this trie.
    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError>;

    /// Returns the value stored at the given deep key.
    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError>;
}

impl<T: BaseRead> TrieQuery for T {
    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::query_value(self.read_root(), HashKey::new(key), self).await
    }

    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        map_base::query_keys_values(self.read_root(), self).await
    }

    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::deep_query_value(self.read_root(), key, self).await
    }
}
