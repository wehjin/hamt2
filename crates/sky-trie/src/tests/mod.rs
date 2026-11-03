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

#[tokio::test]
async fn empty_trie_root_is_empty() {
    let trie = SkyTrieMut::new();
    assert_eq!(MapBase::empty(), trie.active_root());
}

#[tokio::test]
async fn root_round_trip_works() {
    let mut edit = SkyTrie::new().into_edit().await;
    let root = one_kv(HashKey::new(7), TrieValue::U32(7)).await;
    edit.push_root(root.clone()).await;
    assert_eq!(root, edit.active_root());
    let view = edit.commit().await;
    assert_eq!(root, view.active_root());
}

#[tokio::test]
async fn snapshot_does_not_see_subsequent_edits() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(7, 7).await;
        Ok(())
    })
    .await
    .unwrap();
    let view = trie.snapshot();
    let root_before = view.active_root();
    trie.edit(async |trie| {
        trie.insert(8, 8).await;
        Ok(())
    })
    .await
    .unwrap();
    assert_ne!(view.active_root(), trie.active_root());
    assert_eq!(view.active_root(), root_before);
}
