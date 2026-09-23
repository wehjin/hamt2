pub mod prelude;
mod trie;
pub mod trie_reader;

pub use trie::*;
pub use trie_reader::TrieReader;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod stream_tests {
    use crate::Trie;
    use futures::StreamExt;
    use sky_types::storage::MemTrieEdit;
    use sky_types::trie::TrieValue;
    use sky_types::trie::{TrieInsert, TrieStream};

    #[tokio::test]
    async fn u32_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemTrieEdit::new());
        trie.insert(1, TrieValue::U32(1)).await?;
        trie.insert(2, TrieValue::U32(2)).await?;
        trie.deep_insert([3, 4], TrieValue::U32(34), false).await?;
        let mut u32s = trie.u32_stream().collect::<Vec<_>>().await;
        u32s.sort_by_key(|(key, _u32)| *key);
        // Map-base values are skipped by the u32 stream.
        assert_eq!(vec![(1, 1), (2, 2)], u32s);
        Ok(())
    }

    #[tokio::test]
    async fn subtrie_stream() -> anyhow::Result<()> {
        let mut trie = Trie::connect(MemTrieEdit::new());
        trie.deep_insert([1, 101], TrieValue::U32(101), false)
            .await?;
        trie.deep_insert([2, 202], TrieValue::U32(202), false)
            .await?;
        trie.insert(3, TrieValue::U32(33)).await?;
        let subtries = trie.subtrie_stream().collect::<Vec<_>>().await;
        assert_eq!(2, subtries.len());
        Ok(())
    }
}
