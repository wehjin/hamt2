use crate::storage::MemEdit;
use crate::trie::{InsertCursor, QueryCursor, TrieInsert, TrieQuery, TrieValue};

#[tokio::test]
async fn single_hop_works() {
    let mut edit = MemEdit::new();

    // Insert at top level and descend.
    edit.insert(33, 1).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));

    // Lower levels do not have higher level values.
    edit.descend_insert(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);

    // Insert at lower level.
    edit.insert(33, 2).await.unwrap();
    edit.insert(34, 2).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(2)));
    assert_eq!(edit.query(34).await.unwrap(), Some(TrieValue::U32(2)));

    // Higher level does not have lower level values.
    edit.ascend();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));
    assert_eq!(edit.query(34).await.unwrap(), None);
}

#[tokio::test]
async fn multi_hop_works() {
    let mut edit = MemEdit::new();
    edit.insert(33, 1).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));
    edit.descend_insert(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.descend_insert(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.insert(33, 3).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(3)));
    edit.ascend();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.ascend();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));
    edit.descend_insert(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.descend_insert(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(3)));
}
