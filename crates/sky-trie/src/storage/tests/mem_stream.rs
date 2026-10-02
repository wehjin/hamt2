use crate::storage::MemLoad;
use crate::storage::load::StoreLoad;
use crate::trie::{InsertCursor, TrieInsert};
use crate::trie::{TrieStream, TrieValue};
use futures::StreamExt;

#[tokio::test]
async fn u32_stream() -> anyhow::Result<()> {
    let mut trie = MemLoad::new();
    trie.edit(async |trie| {
        trie.insert(1, TrieValue::U32(1)).await?;
        trie.insert(2, TrieValue::U32(2)).await?;
        trie.insert_deep([3, 4], TrieValue::U32(34), false).await?;
        Ok(())
    })
    .await?;
    let mut u32s = trie.u32_stream().collect::<Vec<_>>().await;
    u32s.sort_by_key(|(key, _u32)| *key);
    // Map-base values are skipped by the u32 stream.
    assert_eq!(vec![(1, 1), (2, 2)], u32s);
    Ok(())
}
