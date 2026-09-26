use crate::storage::MemLoad;
use crate::storage::load::StoreLoad;
use crate::trie::{TrieInsert, TrieQuery, TrieSnap, TrieStream, TrieValue, TrieWalk};
use futures::StreamExt;

#[tokio::test]
async fn view_has_query() {
    let mem = MemLoad::new().snapshot();
    let values = mem.query_all().await.unwrap();
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn view_has_snap() {
    let mem = MemLoad::new().snapshot();
    let values = mem.snapshot().query_all().await.unwrap();
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn view_has_stream() {
    let mem = MemLoad::new().snapshot();
    let values = mem.u32_stream().collect::<Vec<_>>().await;
    assert_eq!(values, vec![]);
}

#[tokio::test]
async fn view_has_walk() {
    let mem = MemLoad::new().snapshot();
    let subtries = mem.subtrie_stream().collect::<Vec<_>>().await;
    assert_eq!(subtries.len(), 0);
}

#[tokio::test]
async fn edit_inserts() {
    let insert_value = TrieValue::U32(34);
    let mut mem = MemLoad::new();
    mem.edit(async |edit| {
        edit.insert(33, insert_value).await?;
        Ok(())
    })
    .await
    .unwrap();
    let query_value = mem.query(33).await.unwrap().unwrap();
    assert_eq!(query_value, insert_value)
}
