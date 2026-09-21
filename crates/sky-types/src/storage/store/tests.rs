use crate::storage::store::traits::{StoreConfig, StoreRead};
use crate::storage::{Store, StoreEditError};
use crate::trie::{HandleTrieConfig, SlotBase, SlotBaseId};
use std::assert_matches;

struct TinyList;
impl StoreConfig for TinyList {
    type TrieConfig = HandleTrieConfig;
    const MAX: usize = 1;
}

#[tokio::test]
async fn plain_default_has_empty_slot_base() {
    let list = Store::<TinyList>::default();
    assert_eq!(
        list.read_base(SlotBaseId(0)).expect("read_base"),
        SlotBase::new(),
    );
}
#[tokio::test]
async fn edit_works() {
    let list = Store::<TinyList>::default();
    let mut edit = list.into_mut();
    // Add a slot_base.
    let first_base = SlotBase::new();
    let first_base_id = edit.add_base(first_base.clone()).expect("commit_base");
    assert_eq!(first_base_id, SlotBaseId(1),);
    assert_eq!(
        edit.read_base(first_base_id).expect("read_base"),
        first_base
    );

    // Add a second one. It should fail.
    let second_base = SlotBase::new();
    let result = edit.add_base(second_base.clone());
    assert_matches!(result, Err(StoreEditError::NoSlotsAvailable));

    // Commit the edit. The change should persist.
    let list = edit.commit();
    assert_eq!(list.max_id(), first_base_id);
}

#[tokio::test]
async fn rewind_works() {
    let list = Store::<TinyList>::default();
    let mut edit = list.into_mut();
    // Add a slot_base.
    let first_base = SlotBase::new();
    let first_base_id = edit.add_base(first_base.clone()).expect("commit_base");
    assert_eq!(first_base_id, SlotBaseId(1),);
    assert_eq!(
        edit.read_base(first_base_id).expect("read_base"),
        first_base
    );
    // Rewind the edit. The change should disappear.
    let list = edit.commit();
    assert_eq!(list.max_id(), first_base_id);
}
