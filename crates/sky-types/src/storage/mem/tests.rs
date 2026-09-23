use crate::storage::MemTrie;
use crate::trie::{TrieInsert, TrieQuery, TrieValue};
use anyhow::anyhow;

#[tokio::test]
async fn main_changes_after_ok_from_edit() {
    let mut trie = MemTrie::new();
    let values = trie.query_all().await.unwrap();
    assert_eq!(values.len(), 0);

    let out = trie
        .edit(async |edit| {
            edit.insert(62, TrieValue::U32(63)).await?;
            Ok(64)
        })
        .await
        .unwrap();
    assert_eq!(out, 64);

    let values = trie.query_all().await.unwrap();
    assert_eq!(values, vec![(62, TrieValue::U32(63))]);
}

#[tokio::test]
async fn main_rewinds_after_err_from_edit() {
    let mut trie = MemTrie::new();
    let values = trie.query_all().await.unwrap();
    assert_eq!(values.len(), 0);

    let out = trie
        .edit::<_, ()>(async |edit| {
            edit.insert(62, TrieValue::U32(63)).await?;
            Err(anyhow!("trouble in edit"))
        })
        .await;
    assert!(out.is_err());

    let values = trie.query_all().await.unwrap();
    assert_eq!(values.len(), 0);
}
