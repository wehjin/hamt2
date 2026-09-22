use crate::TrieReader;
use crate::{Trie, TrieQuery};
use futures::Stream;
use futures::stream::StreamExt;
use sky_types::storage::{ReadStorage, ReadWriteStorage, Storage};
use sky_types::trie::map_base::kv_stream;
use sky_types::trie::{MapBase, TrieRead, TrieValue};

impl<S: ReadWriteStorage> TrieQuery for Trie<S> {
    type Subtrie = TrieReader<S::Snapshot>;

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        u32_stream(self.read_root(), self.storage())
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)> {
        subtrie_stream(self.read_root(), self.storage())
    }

    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieReader<S::Snapshot>> {
        TrieReader::subtrie_from_value(value, self.storage().snapshot())
    }
}

impl<S: ReadStorage + TrieRead> TrieQuery for TrieReader<S> {
    type Subtrie = TrieReader<S::Snapshot>;

    fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        u32_stream(self.read_root(), self.storage())
    }

    fn subtrie_stream(&self) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)> {
        subtrie_stream(self.read_root(), self.storage())
    }

    fn to_subtrie_from_value(&self, value: TrieValue) -> Option<TrieReader<S::Snapshot>> {
        TrieReader::subtrie_from_value(value, self.storage().snapshot())
    }
}

fn u32_stream<S>(root: MapBase, storage: &S) -> impl Stream<Item = (i32, u32)>
where
    S: ReadStorage + TrieRead,
{
    let stream = kv_stream(root.clone(), storage.snapshot());
    stream.filter_map(|(key, value)| async move {
        if let TrieValue::U32(val) = value {
            Some((key, val))
        } else {
            None
        }
    })
}

fn subtrie_stream<S>(
    root: MapBase,
    storage: &S,
) -> impl Stream<Item = (i32, TrieReader<S::Snapshot>)>
where
    S: ReadStorage + TrieRead,
{
    let stream = kv_stream(root.clone(), storage.snapshot());
    stream.filter_map(move |(key, value)| {
        let storage = storage.snapshot();
        async move { TrieReader::subtrie_from_value(value, storage).map(|subtrie| (key, subtrie)) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Trie;
    use futures::StreamExt;
    use sky_types::storage::MemStorage;

    #[tokio::test]
    async fn u32_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemStorage::new());
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
        let mut trie = Trie::connect(MemStorage::new());
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
