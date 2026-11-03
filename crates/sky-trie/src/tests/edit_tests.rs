use crate::{SkyTrie, TrieValue};

#[test]
fn new_trie_is_empty() {
    let trie = SkyTrie::new();
    assert_eq!(trie.query_all(), vec![]);
}

#[test]
fn insert_then_query() {
    let insert_value = TrieValue::U32(34);
    let trie = SkyTrie::new().insert(33, insert_value.clone());
    let query_value = trie.query(33).unwrap();
    assert_eq!(query_value, insert_value);
}

#[test]
fn multiple_inserts_accumulate() {
    let trie = SkyTrie::new()
        .insert(20, TrieValue::U32(21))
        .insert(30, TrieValue::U32(31));
    let mut values = trie.query_all();
    values.sort_by(|(a, _), (b, _)| a.cmp(b));
    assert_eq!(
        values,
        vec![(20, TrieValue::U32(21)), (30, TrieValue::U32(31))]
    );
}
