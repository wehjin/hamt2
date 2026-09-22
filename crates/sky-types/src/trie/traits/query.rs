use crate::trie::TrieQueryError;
use crate::trie::TrieValue;
use crate::trie::{HashKey, TrieRead, map_base};
use futures::Stream;

#[allow(async_fn_in_trait)]
pub trait TrieBasicQuery: TrieRead {
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

impl<T: TrieRead> TrieBasicQuery for T {
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

#[allow(async_fn_in_trait)]
pub trait TrieQuery: TrieBasicQuery {
    /// Type produced when a sub-trie is reached in the key-value stream.
    type Subtrie: TrieQuery;

    /// A stream of all `U32` values in this trie, skipping map-base values.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;

    /// A stream of all the sub-tries in this trie.
    fn subtrie_stream(&self) -> impl Stream<Item = (i32, Self::Subtrie)>;

    /// Converts a map-base value into a sub-trie.
    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<Self::Subtrie>;
}
