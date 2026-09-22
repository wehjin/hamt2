use crate::storage::store::traits::StoreConfig;
use crate::storage::{Store, StoreEditError, StoreRead};
use crate::trie::{SlotBase, SlotBaseId, TrieBasicQuery, TrieRead};
use std::assert_matches;

struct TinyStore;
impl StoreConfig for TinyStore {
    const MAX: usize = 1;
}

struct BigStore;
impl StoreConfig for BigStore {
    const MAX: usize = 10000;
}

#[tokio::test]
async fn inherits_trie_query() {
    let store = Store::<BigStore>::default();
    let value = store.query_value(33).await.expect("query value");
    assert_eq!(value, None);

    let edit = store.into_mut();
    let value_while_editing = edit.query_value(12).await.expect("query value");
    assert_eq!(value_while_editing, None);
}

#[tokio::test]
async fn plain_default_has_empty_slot_base() {
    let store = Store::<TinyStore>::default();
    assert_eq!(
        store.read_base(SlotBaseId(0)).await.expect("read_base"),
        SlotBase::empty(),
    );
}
#[tokio::test]
async fn edit_works() {
    let store = Store::<TinyStore>::default();
    let mut edit = store.into_mut();
    // Add a slot_base.
    let first_base = SlotBase::empty();
    let first_base_id = edit.add_base(first_base.clone()).expect("commit_base");
    assert_eq!(first_base_id, SlotBaseId(1),);
    assert_eq!(
        edit.read_base(first_base_id).await.expect("read_base"),
        first_base
    );

    // Add a second one. It should fail.
    let second_base = SlotBase::empty();
    let result = edit.add_base(second_base.clone());
    assert_matches!(result, Err(StoreEditError::NoSlotsAvailable));

    // Commit the edit. The change should persist.
    let back_to_store = edit.commit();
    assert_eq!(back_to_store.max_id(), first_base_id);
}

#[tokio::test]
async fn rewind_works() {
    let store = Store::<TinyStore>::default();
    let mut edit = store.into_mut();
    // Add a slot_base.
    let first_base = SlotBase::empty();
    let first_base_id = edit.add_base(first_base.clone()).expect("commit_base");
    assert_eq!(first_base_id, SlotBaseId(1),);
    assert_eq!(
        edit.read_base(first_base_id).await.expect("read_base"),
        first_base
    );
    // Rewind the edit. The change should disappear.
    let back_to_store = edit.commit();
    assert_eq!(back_to_store.max_id(), first_base_id);
}
