use crate::QueryError;
use crate::trie::base_storage::BaseStorageRead;
use crate::trie::core::deep_key::DeepKey;
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::trie::trie_ref::TrieRef;
use futures::Stream;
use futures::stream::StreamExt;

/// The read-only query interface shared by [`Trie`], [`TrieRef`], and
/// [`ReadTrie`](crate::trie::ReadTrie).
///
/// Every query method is provided by default; implementations only need to
/// expose the root map base and the storage.
#[allow(async_fn_in_trait)]
pub trait TrieQuery<S: BaseStorageRead> {
    /// The root map base of this trie.
    fn root(&self) -> &MapBase;

    /// The storage this trie reads bases from.
    fn storage(&self) -> &S;

    /// Returns the value stored at the given key or none if the key is absent.
    async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        self.root()
            .query_value(TrieKey::new(key), self.storage())
            .await
    }

    /// Returns all keys and values in this trie.
    async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.root().query_keys_values(self.storage()).await
    }

    /// Returns the value stored at the given deep key.
    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<MemValue>, QueryError> {
        let deep_key = DeepKey::from(key);
        let mut current_map_base = self.root().clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match current_map_base
                .query_value(deep_key[i].clone(), self.storage())
                .await?
            {
                None => {
                    return Ok(None);
                }
                Some(value) => {
                    if i < last_index {
                        let MemValue::MapBase(map_base) = value else {
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
        let stream = self.root().clone().kv_stream(self.storage());
        stream.filter_map(|(key, value)| async move {
            if let MemValue::U32(val) = value {
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
        let stream = self.root().clone().kv_stream(self.storage());
        stream.filter_map(move |(key, value)| async move {
            TrieRef::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie))
        })
    }

    /// Converts a map-base value into a borrowed sub-trie over the same storage.
    fn to_subtrie_from_value(&self, value: MemValue) -> Option<TrieRef<'_, S>> {
        TrieRef::subtrie_from_value(value, self.storage())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::Trie;
    use crate::trie::base_storage::mem::MemBaseStorage;
    use futures::StreamExt;

    #[tokio::test]
    async fn u32_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemBaseStorage::new()).await?;
        trie = trie.insert(1, MemValue::U32(1)).await?;
        trie = trie.insert(2, MemValue::U32(2)).await?;
        trie = trie.deep_insert([3, 4], MemValue::U32(34), false).await?;
        let mut u32s = trie.u32_stream().collect::<Vec<_>>().await;
        u32s.sort_by_key(|(key, _u32)| *key);
        // Map-base values are skipped by the u32 stream.
        assert_eq!(vec![(1, 1), (2, 2)], u32s);
        Ok(())
    }

    #[tokio::test]
    async fn subtrie_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemBaseStorage::new()).await?;
        trie = trie
            .deep_insert([1, 101], MemValue::U32(101), false)
            .await?;
        trie = trie
            .deep_insert([2, 202], MemValue::U32(202), false)
            .await?;
        trie = trie.insert(3, MemValue::U32(33)).await?;
        let subtries = trie.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(2, subtries.len());
        Ok(())
    }
}
