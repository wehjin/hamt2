use crate::TrieQuery;
use crate::TrieReader;
use crate::crate_services::map_base::{kv_stream, query_keys_values, query_value};
use crate::trie_storage::{ReadTrieStorage, ReadWriteTrieStorage};
use crate::types::DeepKey;
use crate::types::HashKey;
use crate::types::trie_value::TrieValue;
use futures::Stream;
use futures::stream::StreamExt;
use sky_types::trie::error::TrieQueryError;
use sky_types::trie::map_base::MapBase;

/// The storage-backed query interface shared by [`Trie`] and
/// [`TrieReader`](crate::TrieReader).
///
/// Implementations only need to expose the storage; the root and every
/// [`TrieQuery`] method are provided by the direct implementations below.
pub trait StorageTrieQuery<S: ReadTrieStorage>: TrieQuery<S> {
    /// The storage this trie reads bases from.
    fn storage(&self) -> &S;
}

impl<S: ReadWriteTrieStorage> TrieQuery<S> for crate::Trie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        query_value(self.root(), HashKey::new(key), self.storage()).await
    }

    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        query_keys_values(self.root(), self.storage()).await
    }

    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        deep_query_value(self.root(), self.storage(), key).await
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        u32_stream(self.root(), self.storage())
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)> {
        subtrie_stream(self.root(), self.storage())
    }

    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieReader<S::Snapshot>> {
        TrieReader::subtrie_from_value(value, self.storage().snapshot())
    }
}

impl<S: ReadTrieStorage> TrieQuery<S> for TrieReader<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    async fn query_value(&self, key: i32) -> Result<Option<TrieValue>, TrieQueryError> {
        query_value(self.root(), HashKey::new(key), self.storage()).await
    }

    async fn query_keys_values(&self) -> Result<Vec<(i32, TrieValue)>, TrieQueryError> {
        query_keys_values(self.root(), self.storage()).await
    }

    async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<TrieValue>, TrieQueryError> {
        deep_query_value(self.root(), self.storage(), key).await
    }

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        u32_stream(self.root(), self.storage())
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)> {
        subtrie_stream(self.root(), self.storage())
    }

    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieReader<S::Snapshot>> {
        TrieReader::subtrie_from_value(value, self.storage().snapshot())
    }
}

async fn deep_query_value<const N: usize, S: ReadTrieStorage>(
    root: &MapBase,
    storage: &S,
    key: [i32; N],
) -> Result<Option<TrieValue>, TrieQueryError> {
    let deep_key = DeepKey::from(key);
    let mut current_map_base = root.clone();
    let last_index = N - 1;
    for i in 0..=last_index {
        match query_value(&current_map_base, deep_key[i].clone(), storage).await? {
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

fn u32_stream<'a, S: ReadTrieStorage>(
    root: &'a MapBase,
    storage: &'a S,
) -> impl Stream<Item = (i32, u32)> {
    let stream = kv_stream(root.clone(), storage.snapshot());
    stream.filter_map(|(key, value)| async move {
        if let TrieValue::U32(val) = value {
            Some((key, val))
        } else {
            None
        }
    })
}

fn subtrie_stream<'a, S: ReadTrieStorage>(
    root: &'a MapBase,
    storage: &'a S,
) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)> {
    let storage = storage.snapshot();
    let stream = kv_stream(root.clone(), storage.clone());
    stream.filter_map(move |(key, value)| {
        let storage = storage.clone();
        async move { TrieReader::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie)) }
    })
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
