use crate::storage::ReadStorage;
use crate::trie::TrieValue;
use crate::trie::map_base::kv_stream;
use crate::trie::{HashKey, TrieRead, map_base};
use crate::trie::{MapBase, TrieQueryError};
use futures::Stream;
use futures::StreamExt;

#[allow(async_fn_in_trait)]
pub trait TrieQuery: TrieRead {
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

impl<T: TrieRead> TrieQuery for T {
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

/// Implement this trait and provide `to_subtrie` to acquire streaming access
/// to stored values.
pub trait TrieStream: ReadStorage {
    type Subtrie: TrieStream;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie;

    fn to_subtrie_in_value(&self, trie_value: TrieValue) -> Option<Self::Subtrie> {
        match trie_value {
            TrieValue::U32(_) => None,
            TrieValue::SubTrie(map_base) => Some(self.to_subtrie(map_base)),
        }
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, Self::Subtrie)> {
        self.map_base_stream()
            .map(|(key, map_base)| (key, self.to_subtrie(map_base)))
    }

    /// Stream all map-base key values in the trie.
    fn map_base_stream(&self) -> impl Stream<Item = (i32, MapBase)> {
        let stream = kv_stream(self.read_root(), self.snapshot());
        stream.filter_map(move |(key, value)| async move {
            if let TrieValue::SubTrie(map_base) = value {
                Some((key, map_base))
            } else {
                None
            }
        })
    }

    /// Stream all u32 keyed values in the trie.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        let stream = kv_stream(self.read_root(), self.snapshot());
        stream.filter_map(|(key, value)| async move {
            if let TrieValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }
}
