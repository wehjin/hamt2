use crate::storage::MemEdit;
use crate::trie::{InsertCursor, QueryCursor, TrieInsert, TrieQuery, TrieValue};

#[tokio::test]
async fn insert_deep() {
    let mut edit = MemEdit::new();
    edit.insert_deep([1, 2, 10], 10, false).await.unwrap();
    edit.insert_deep([1, 2, 11], 11, false).await.unwrap();
    edit.descend_keys([1, 2]).await.unwrap();
    assert_eq!(edit.query_u32(10).await.unwrap(), Some(10));
    assert_eq!(edit.query_u32(11).await.unwrap(), Some(11));
    edit.ascend_n(2);

    let backup = edit.backup();
    edit.insert_deep([1, 2, 12], 12, true).await.unwrap();
    edit.descend_keys([1, 2]).await.unwrap();
    assert_eq!(edit.query_u32(10).await.unwrap(), None);
    assert_eq!(edit.query_u32(11).await.unwrap(), None);
    assert_eq!(edit.query_u32(12).await.unwrap(), Some(12));

    edit.restore(backup);
    edit.descend_keys([1, 2]).await.unwrap();
    assert_eq!(edit.query_u32(10).await.unwrap(), Some(10));
    assert_eq!(edit.query_u32(11).await.unwrap(), Some(11));
    assert_eq!(edit.query_u32(12).await.unwrap(), None);
}

#[tokio::test]
async fn single_hop_works() {
    let mut edit = MemEdit::new();

    // Insert at top level and descend.
    edit.insert(33, 1).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));

    // Lower levels do not have higher level values.
    edit.descend(40).await.unwrap();
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
    edit.descend(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.descend(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.insert(33, 3).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(3)));
    edit.ascend();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.ascend();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(1)));
    edit.descend(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), None);
    edit.descend(40).await.unwrap();
    assert_eq!(edit.query(33).await.unwrap(), Some(TrieValue::U32(3)));
}
