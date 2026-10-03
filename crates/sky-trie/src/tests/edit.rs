use crate::SkyMapMut;
use crate::{InsertCursor, Insert, Query, TrieValue};

#[tokio::test]
async fn query_value_exists_works() {
    let mut m = SkyMapMut::new();
    m.insert(32, TrieValue::U32(33)).await;
    let v = m.query(32).await;
    assert_eq!(v, Some(TrieValue::U32(33)));
}

#[tokio::test]
async fn deep_insert_and_query_works() {
    let mut m = SkyMapMut::new();
    m.insert_deep([1, 2, 3], 45, false).await;
    let v = m.query_deep([1, 2, 3]).await;
    assert_eq!(v, Some(TrieValue::U32(45)));
}
