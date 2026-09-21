use crate::trie::TrieQueryError;
use crate::trie::TrieValue;
use crate::trie::{HashKey, MapBase, TrieBaseRead, map_base};
use futures::Stream;

pub trait RootTrieQuery {
    /// The root map base of this trie.
    fn root(&self) -> MapBase;
}

#[allow(async_fn_in_trait)]
pub trait ShallowTrieQuery: RootTrieQuery {
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

impl<T: RootTrieQuery + TrieBaseRead> ShallowTrieQuery for T {
    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::query_value(self.root(), HashKey::new(key), self).await
    }

    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        map_base::query_keys_values(self.root(), self).await
    }

    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        map_base::deep_query_value(self.root(), key, self).await
    }
}

/// The read-only query interface shared by every storage-backed trie.
///
/// All methods are required; the storage-backed types implement this directly.
/// `Subtrie` is the type of a queryable view over a sub-trie, so callers of
/// `subtrie_stream()` and `to_subtrie_from_value()` never need to name the
/// concrete reader type or a storage type.
#[allow(async_fn_in_trait)]
pub trait TrieQuery: ShallowTrieQuery {
    /// The type of a queryable view over a sub-trie.
    type Subtrie: TrieQuery;

    /// A stream of all `U32` values in this trie, skipping map-base values.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;

    /// A stream of all the sub-tries in this trie.
    fn subtrie_stream(&self) -> impl Stream<Item = (i32, Self::Subtrie)>;

    /// Converts a map-base value into a sub-trie.
    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<Self::Subtrie>;
}
