use crate::SkyTrieMut;
use crate::{Insert, InsertCursor, Query, QueryCursor, TrieValue};

#[test]
fn insert_deep() {
    let mut edit = SkyTrieMut::new();
    edit.insert_deep([1, 2, 10], 10, false);
    edit.insert_deep([1, 2, 11], 11, false);
    edit.descend_n([1, 2]);
    assert_eq!(edit.query_u32(10), Some(10));
    assert_eq!(edit.query_u32(11), Some(11));
    edit.ascend_n(2);

    let backup = edit.backup();
    edit.insert_deep([1, 2, 12], 12, true);
    edit.descend_n([1, 2]);
    assert_eq!(edit.query_u32(10), None);
    assert_eq!(edit.query_u32(11), None);
    assert_eq!(edit.query_u32(12), Some(12));

    edit.restore(backup);
    edit.descend_n([1, 2]);
    assert_eq!(edit.query_u32(10), Some(10));
    assert_eq!(edit.query_u32(11), Some(11));
    assert_eq!(edit.query_u32(12), None);
}

#[test]
fn single_hop_works() {
    let mut edit = SkyTrieMut::new();

    // Insert at top level and descend.
    edit.insert(33, 1);
    assert_eq!(edit.query(33), Some(TrieValue::U32(1)));

    // Lower levels do not have higher level values.
    edit.descend(40);
    assert_eq!(edit.query(33), None);

    // Insert at lower level.
    edit.insert(33, 2);
    edit.insert(34, 2);
    assert_eq!(edit.query(33), Some(TrieValue::U32(2)));
    assert_eq!(edit.query(34), Some(TrieValue::U32(2)));

    // Higher level does not have lower level values.
    edit.ascend();
    assert_eq!(edit.query(33), Some(TrieValue::U32(1)));
    assert_eq!(edit.query(34), None);
}

#[test]
fn multi_hop_works() {
    let mut edit = SkyTrieMut::new();
    edit.insert(33, 1);
    assert_eq!(edit.query(33), Some(TrieValue::U32(1)));
    edit.descend(40);
    assert_eq!(edit.query(33), None);
    edit.descend(40);
    assert_eq!(edit.query(33), None);
    edit.insert(33, 3);
    assert_eq!(edit.query(33), Some(TrieValue::U32(3)));
    edit.ascend();
    assert_eq!(edit.query(33), None);
    edit.ascend();
    assert_eq!(edit.query(33), Some(TrieValue::U32(1)));
    edit.descend(40);
    assert_eq!(edit.query(33), None);
    edit.descend(40);
    assert_eq!(edit.query(33), Some(TrieValue::U32(3)));
}
