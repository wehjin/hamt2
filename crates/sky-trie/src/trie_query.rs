use crate::TrieReader;
use crate::trie_storage::ReadTrieStorage;
use crate::types::trie_value::TrieValue;
use futures::Stream;
use sky_types::trie::error::TrieQueryError;
use sky_types::trie::map_base::MapBase;

/// The query interface shared by [`Trie`] and [`TrieReader`](crate::TrieReader).
///
/// All methods are required; the storage-backed types implement this directly.
#[allow(async_fn_in_trait)]
pub trait TrieQuery<S: ReadTrieStorage> {
    /// The root map base of this trie.
    fn root(&self) -> &MapBase;

    /// Returns the value stored at the given key or none if the key is absent.
    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError>;

    /// Returns all keys and values in this trie.
    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError>;

    /// Returns the value stored at the given deep key.
    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError>;

    /// A stream of all `U32` values in this trie, skipping map-base values.
    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)>;

    /// A stream of all the sub-tries in this trie.
    fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)>;

    /// Converts a map-base value into a sub-trie over a snapshot of the storage.
    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieReader<S::Snapshot>>;
}
