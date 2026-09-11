use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadWrite};
use crate::trie::core::key::TrieKey;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::Trie;
use crate::QueryError;
use futures::Stream;
use futures::stream::StreamExt;

/// A borrowed, read-only view of a trie over a storage.
#[derive(Debug, Clone)]
pub struct TrieRef<'a, S: BaseStorageRead> {
    root: MapBase,
    storage: &'a S,
}

impl<'a, S: BaseStorageRead> TrieRef<'a, S> {
    pub fn new(root: MapBase, storage: &'a S) -> Self {
        Self { root, storage }
    }

    pub async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        self.root.query_value(key, self.storage).await
    }

    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.root.query_keys_values(self.storage).await
    }

    pub fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> + 'a {
        let stream = self.root.clone().kv_stream(self.storage);
        stream.filter_map(|(key, value)| async move {
            if let MemValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }

    pub fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieRef<'a, S>)> + 'a {
        let storage = self.storage;
        let stream = self.root.clone().kv_stream(self.storage);
        stream.filter_map(move |(key, value)| async move {
            Self::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie))
        })
    }

    /// Converts a map-base value into a sub-trie view over the same storage.
    pub fn to_subtrie_from_value(&self, value: MemValue) -> Option<Self> {
        Self::subtrie_from_value(value, self.storage)
    }

    pub fn subtrie_from_value(value: MemValue, storage: &'a S) -> Option<Self> {
        let root = match value {
            MemValue::MapBase(root) => root,
            MemValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }
}

impl<S: BaseStorageReadWrite> Trie<S> {
    /// A borrowed read-only view of this trie.
    pub fn view(&self) -> TrieRef<'_, S> {
        TrieRef::new(self.root.clone(), &self.storage)
    }

    pub async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        self.view().query_value(key).await
    }

    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.view().query_keys_values().await
    }

    pub fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        let stream = self.root.clone().kv_stream(&self.storage);
        stream.filter_map(|(key, value)| async move {
            if let MemValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }

    pub fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieRef<'_, S>)> {
        let storage = &self.storage;
        let stream = self.root.clone().kv_stream(storage);
        stream.filter_map(move |(key, value)| async move {
            TrieRef::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie))
        })
    }

    /// Converts a map-base value into a sub-trie view over the same storage.
    pub fn to_subtrie_from_value(&self, value: MemValue) -> Option<TrieRef<'_, S>> {
        self.view().to_subtrie_from_value(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        trie = trie.deep_insert([1, 101], MemValue::U32(101), false).await?;
        trie = trie.deep_insert([2, 202], MemValue::U32(202), false).await?;
        trie = trie.insert(3, MemValue::U32(33)).await?;
        let subtries = trie.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(2, subtries.len());
        Ok(())
    }
}
