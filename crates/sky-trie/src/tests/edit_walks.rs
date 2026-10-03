use crate::SkyTrieMut;
use crate::{InsertCursor, QueryCursor, Insert, Query, TrieValue};

#[tokio::test]
async fn insert_deep() {
    let mut edit = SkyTrieMut::new();
    edit.insert_deep([1, 2, 10], 10, false).await;
    edit.insert_deep([1, 2, 11], 11, false).await;
    edit.descend_n([1, 2]).await;
    assert_eq!(edit.query_u32(10).await, Some(10));
    assert_eq!(edit.query_u32(11).await, Some(11));
    edit.ascend_n(2);

    let backup = edit.backup();
    edit.insert_deep([1, 2, 12], 12, true).await;
    edit.descend_n([1, 2]).await;
    assert_eq!(edit.query_u32(10).await, None);
    assert_eq!(edit.query_u32(11).await, None);
    assert_eq!(edit.query_u32(12).await, Some(12));

    edit.restore(backup);
    edit.descend_n([1, 2]).await;
    assert_eq!(edit.query_u32(10).await, Some(10));
    assert_eq!(edit.query_u32(11).await, Some(11));
    assert_eq!(edit.query_u32(12).await, None);
}

#[tokio::test]
async fn single_hop_works() {
    let mut edit = SkyTrieMut::new();

    // Insert at top level and descend.
    edit.insert(33, 1).await;
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(1)));

    // Lower levels do not have higher level values.
    edit.descend(40).await;
    assert_eq!(edit.query(33).await, None);

    // Insert at lower level.
    edit.insert(33, 2).await;
    edit.insert(34, 2).await;
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(2)));
    assert_eq!(edit.query(34).await, Some(TrieValue::U32(2)));

    // Higher level does not have lower level values.
    edit.ascend();
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(1)));
    assert_eq!(edit.query(34).await, None);
}

#[tokio::test]
async fn multi_hop_works() {
    let mut edit = SkyTrieMut::new();
    edit.insert(33, 1).await;
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(1)));
    edit.descend(40).await;
    assert_eq!(edit.query(33).await, None);
    edit.descend(40).await;
    assert_eq!(edit.query(33).await, None);
    edit.insert(33, 3).await;
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(3)));
    edit.ascend();
    assert_eq!(edit.query(33).await, None);
    edit.ascend();
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(1)));
    edit.descend(40).await;
    assert_eq!(edit.query(33).await, None);
    edit.descend(40).await;
    assert_eq!(edit.query(33).await, Some(TrieValue::U32(3)));
}
