use crate::{SkyTrie, TrieValue};

mod edit;
mod edit_tests;
mod mem_insert;

#[test]
fn insert_does_not_affect_original() {
    let trie = SkyTrie::new();
    let next = trie.insert(7, 7);
    assert_eq!(trie.query(7), None);
    assert_eq!(next.query(7), Some(TrieValue::U32(7)));
}
