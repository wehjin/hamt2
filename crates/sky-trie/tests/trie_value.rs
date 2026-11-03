use sky_trie::{SkyTrie, TrieValue};

fn as_trie(value: TrieValue) -> SkyTrie {
    let TrieValue::Trie(trie) = value else {
        panic!("expected Trie");
    };
    trie
}

#[test]
fn query_passes_through_trie() {
    let value = TrieValue::Trie(SkyTrie::new().insert(7, 42u32));
    assert_eq!(value.query(7), Some(TrieValue::U32(42)));
    assert_eq!(value.query(8), None);
}

#[test]
fn query_deep_passes_through_trie() {
    let value = TrieValue::Trie(SkyTrie::new().insert_deep([1, 2, 3], 42u32));
    assert_eq!(value.query_deep([1, 2, 3]), Some(TrieValue::U32(42)));
    assert_eq!(
        value.query_deep([1, 2]),
        Some(TrieValue::Trie(SkyTrie::new().insert(3, 42u32)))
    );
    assert_eq!(value.query_deep([1, 9]), None);
}

#[test]
fn query_on_leaf_returns_none() {
    assert_eq!(TrieValue::U32(42).query(0), None);
    assert_eq!(TrieValue::Bytes(vec![1, 2]).query(0), None);
    assert_eq!(TrieValue::Nil.query(0), None);
}

#[test]
fn query_deep_on_leaf_returns_none() {
    assert_eq!(TrieValue::U32(42).query_deep([1, 2]), None);
    assert_eq!(TrieValue::Bytes(vec![1, 2]).query_deep([1, 2]), None);
    assert_eq!(TrieValue::Nil.query_deep([1, 2, 3]), None);
}

#[test]
fn insert_on_nil_promotes_to_trie() {
    let value = TrieValue::Nil.insert(7, 42u32);
    assert_eq!(as_trie(value).query(7), Some(TrieValue::U32(42)));
}

#[test]
fn insert_on_trie_passes_through_and_preserves_original() {
    let base = SkyTrie::new().insert(1, 10u32);
    let value = TrieValue::Trie(base.clone());
    let out = value.insert(2, 20u32);
    assert_eq!(base.query(2), None);
    let trie = as_trie(out);
    assert_eq!(trie.query(1), Some(TrieValue::U32(10)));
    assert_eq!(trie.query(2), Some(TrieValue::U32(20)));
}

#[test]
fn insert_overwrites_existing_key() {
    let value = TrieValue::Trie(SkyTrie::new().insert(1, 10u32));
    let trie = as_trie(value.insert(1, 99u32));
    assert_eq!(trie.query(1), Some(TrieValue::U32(99)));
}

#[test]
fn insert_deep_on_nil_promotes_to_trie() {
    let value = TrieValue::Nil.insert_deep([1, 2, 3], 42u32);
    assert_eq!(
        as_trie(value).query_deep([1, 2, 3]),
        Some(TrieValue::U32(42))
    );
}

#[test]
fn insert_deep_on_trie_passes_through_and_preserves_original() {
    let base = SkyTrie::new().insert_deep([1, 2], 10u32);
    let value = TrieValue::Trie(base.clone());
    let out = value.insert_deep([1, 3], 20u32);
    assert_eq!(base.query_deep([1, 3]), None);
    let trie = as_trie(out);
    assert_eq!(trie.query_deep([1, 2]), Some(TrieValue::U32(10)));
    assert_eq!(trie.query_deep([1, 3]), Some(TrieValue::U32(20)));
}

#[test]
fn insert_accepts_into_types() {
    let value = TrieValue::Nil.insert(7, 42u32).insert(8, "hello");
    let trie = as_trie(value);
    assert_eq!(trie.query(7), Some(TrieValue::U32(42)));
    assert_eq!(trie.query(8), Some(TrieValue::Bytes(b"hello".to_vec())));
}

#[test]
#[should_panic(expected = "leaf")]
fn insert_on_u32_panics() {
    let _ = TrieValue::U32(42).insert(1, 1u32);
}

#[test]
#[should_panic(expected = "leaf")]
fn insert_on_bytes_panics() {
    let _ = TrieValue::Bytes(vec![1, 2]).insert(1, 1u32);
}

#[test]
#[should_panic(expected = "leaf")]
fn insert_deep_on_u32_panics() {
    let _ = TrieValue::U32(42).insert_deep([1, 2], 1u32);
}

#[test]
#[should_panic(expected = "leaf")]
fn insert_deep_on_bytes_panics() {
    let _ = TrieValue::Bytes(vec![1, 2]).insert_deep([1], 1u32);
}

#[test]
fn stored_nil_is_some_nil_not_none() {
    let trie = SkyTrie::new().insert(7, TrieValue::Nil);
    let value = TrieValue::Trie(trie);
    assert_eq!(value.query(7), Some(TrieValue::Nil));
    assert_eq!(value.query(8), None);
}

#[test]
fn nil_behaves_as_empty_trie() {
    assert_eq!(TrieValue::Nil.query(1), None);
    let trie = as_trie(TrieValue::Nil.insert(1, 1u32));
    assert_eq!(trie.query(1), Some(TrieValue::U32(1)));
}
