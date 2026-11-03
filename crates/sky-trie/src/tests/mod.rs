use crate::Insert;
use crate::QueryCursor;
use crate::services::map_base::one_kv;
use crate::{HashKey, MapBase, TrieValue};
use crate::{SkyTrie, SkyTrieMut};
use crate::Snap;

mod edit;
mod edit_tests;
mod edit_walks;
mod mem_insert;

#[test]
fn empty_trie_root_is_empty() {
    let trie = SkyTrieMut::new();
    assert_eq!(MapBase::empty(), trie.active_root());
}

#[test]
fn root_round_trip_works() {
    let mut edit = SkyTrie::new().into_edit();
    let root = one_kv(HashKey::new(7), TrieValue::U32(7));
    edit.push_root(root.clone());
    assert_eq!(root, edit.active_root());
    let view = edit.commit();
    assert_eq!(root, view.active_root());
}

#[test]
fn snapshot_does_not_see_subsequent_edits() {
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        trie.insert(7, 7);
        Ok(())
    })
    .unwrap();
    let view = trie.snapshot();
    let root_before = view.active_root();
    trie.edit(|trie| {
        trie.insert(8, 8);
        Ok(())
    })
    .unwrap();
    assert_ne!(view.active_root(), trie.active_root());
    assert_eq!(view.active_root(), root_before);
}
