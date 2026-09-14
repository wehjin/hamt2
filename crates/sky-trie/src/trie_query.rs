use crate::TrieQueryError;
use crate::trie_ref::TrieRef;
use crate::trie_storage::ReadTrieStorage;
use crate::types::DeepKey;
use crate::types::HashKey;
use crate::types::map_base::{MapBase, kv_stream, query_keys_values, query_value};
use crate::types::trie_value::TrieValue;
use futures::Stream;
use futures::stream::StreamExt;

/// The read-only query interface shared by [`Trie`], [`TrieRef`], and
/// [`ReadTrie`](crate::TrieReader).
///
/// Every query method is provided by default; implementations only need to
/// expose the root map base and the storage.
#[allow(async_fn_in_trait)]
pub trait TrieQuery<S: ReadTrieStorage> {
    /// The root map base of this trie.
    fn root(&self) -> &MapBase;

    /// The storage this trie reads bases from.
    fn storage(&self) -> &S;

    /// Returns the value stored at the given key or none if the key is absent.
    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        query_value(self.root(), HashKey::new(key), self.storage()).await
    }

    /// Returns all keys and values in this trie.
    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        query_keys_values(self.root(), self.storage()).await
    }

    /// Returns the value stored at the given deep key.
    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        let deep_key = DeepKey::from(key);
        let mut current_map_base = self.root().clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match query_value(&current_map_base, deep_key[i].clone(), self.storage()).await? {
                None => {
                    return Ok(None);
                }
                Some(value) => {
                    if i < last_index {
                        let TrieValue::SubTrie(map_base) = value else {
                            // A non-map value has no sub-trie below it.
                            return Ok(None);
                        };
                        current_map_base = map_base;
                    } else {
                        return Ok(Some(value));
                    }
                }
            }
        }
        unreachable!();
    }

    /// A stream of all `U32` values in this trie, skipping map-base values.
    fn u32_stream<'a>(&'a self) -> impl Stream<Item = (i32, u32)> + 'a
    where
        S: 'a,
    {
        let stream = kv_stream(self.root().clone(), self.storage());
        stream.filter_map(|(key, value)| async move {
            if let TrieValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }

    /// A stream of all the sub-tries in this trie.
    fn subtrie_stream<'a>(&'a self) -> impl Stream<Item = (i32, TrieRef<'a, S>)> + 'a
    where
        S: 'a,
    {
        let storage = self.storage();
        let stream = kv_stream(self.root().clone(), self.storage());
        stream.filter_map(move |(key, value)| async move {
            TrieRef::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie))
        })
    }

    /// Converts a map-base value into a borrowed sub-trie over the same storage.
    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieRef<'_, S>> {
        TrieRef::subtrie_from_value(value, self.storage())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Trie;
    use crate::trie_storage::mem::MemTrieStorage;
    use futures::StreamExt;

    #[tokio::test]
    async fn u32_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemTrieStorage::new()).await?;
        trie = trie.insert(1, TrieValue::U32(1)).await?;
        trie = trie.insert(2, TrieValue::U32(2)).await?;
        trie = trie.deep_insert([3, 4], TrieValue::U32(34), false).await?;
        let mut u32s = trie.u32_stream().collect::<Vec<_>>().await;
        u32s.sort_by_key(|(key, _u32)| *key);
        // Map-base values are skipped by the u32 stream.
        assert_eq!(vec![(1, 1), (2, 2)], u32s);
        Ok(())
    }

    #[tokio::test]
    async fn subtrie_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemTrieStorage::new()).await?;
        trie = trie
            .deep_insert([1, 101], TrieValue::U32(101), false)
            .await?;
        trie = trie
            .deep_insert([2, 202], TrieValue::U32(202), false)
            .await?;
        trie = trie.insert(3, TrieValue::U32(33)).await?;
        let subtries = trie.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(2, subtries.len());
        Ok(())
    }
}
