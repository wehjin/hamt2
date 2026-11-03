use crate::{SkyTrie, TrieValue};

#[test]
fn query_value_exists_works() {
    let m = SkyTrie::new().insert(32, TrieValue::U32(33));
    let v = m.query(32);
    assert_eq!(v, Some(TrieValue::U32(33)));
}

#[test]
fn deep_insert_and_query_works() {
    let m = SkyTrie::new().insert_deep([1, 2, 3], 45);
    let v = m.query_deep([1, 2, 3]);
    assert_eq!(v, Some(TrieValue::U32(45)));
}
