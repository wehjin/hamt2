use crate::space::{Read, Space};
use crate::trie::core::key::TrieKey;
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::QueryError;
use futures::Stream;
use std::rc::Rc;

impl<T: Space> SpaceTrie<T> {
    pub async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        self.map_base.query_value(key, &self.reader).await
    }

    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.map_base.query_keys_values(&self.reader).await
    }

    pub fn filter_map<V, Fut: Future<Output = Option<V>>>(
        &self,
        reader: &impl Read,
        filter: impl Fn((i32, MemValue)) -> Fut,
    ) -> impl Stream<Item = V> {
        use futures::stream::StreamExt;
        let filter = Rc::new(filter);
        let stream = self.map_base.kv_stream(reader);
        stream.filter_map(move |kv| {
            let filter = filter.clone();
            async move {
                let output = filter(kv).await;
                output
            }
        })
    }
    pub fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.filter_map(&self.reader, |(key, value)| async move {
            if let MemValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }

    pub fn subtrie_stream(&self) -> impl Stream<Item = (i32, SpaceTrie<T>)> {
        let clone_reader = Rc::new(|| self.reader.clone());
        self.filter_map(&self.reader, move |(key, value)| {
            let reader_source = clone_reader.clone();
            async move {
                Self::subtrie_from_value(value, reader_source())
                    .await
                    .ok()
                    .map(|subtrie| (key, subtrie))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::file::FileSpace;
    use crate::space::mem::MemSpace;
    use futures::StreamExt;

    #[tokio::test]
    async fn u32_stream() -> anyhow::Result<()> {
        let mut space = MemSpace::new();
        {
            let mut trie = SpaceTrie::connect(&space).await?;
            trie = trie.insert(1, MemValue::U32(1)).await?;
            trie = trie.insert(2, MemValue::U32(2)).await?;
            trie = {
                let subtrie = trie.new_subtrie();
                trie.insert(3, MemValue::MapBase(subtrie.unwrap())).await?
            };
            trie.commit(&mut space).await?;
        }
        let trie = SpaceTrie::connect(&space).await?;
        let mut u32s = trie.u32_stream().collect::<Vec<_>>().await;
        u32s.sort_by_key(|(key, _u32)| *key);
        // The subtrie turns into an integer after the commit.
        assert_eq!(vec![(1, 1), (2, 2), (3, 0)], u32s);
        Ok(())
    }
    #[tokio::test]
    async fn subtrie_stream() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        {
            let mut space = FileSpace::new(&file).await?;
            let mut trie = SpaceTrie::connect(&space).await?;
            trie = {
                let subtrie = trie.new_subtrie();
                trie.insert(1, MemValue::MapBase(subtrie.unwrap())).await?
            };
            trie = {
                let subtrie = trie.new_subtrie();
                trie.insert(2, MemValue::MapBase(subtrie.unwrap())).await?
            };
            trie.commit(&mut space).await?;
        }
        let space = FileSpace::load(&file).await?;
        let trie = SpaceTrie::connect(&space).await?;
        let subtries = trie.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(2, subtries.len());
        Ok(())
    }
}
