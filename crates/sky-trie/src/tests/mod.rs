use crate::Snap;
use crate::services::map_base::one_kv;
use crate::{Base, Buffer, BufferIndex, BufferMut, HashKey, MapBase, TrieValue};
use crate::{SkyTrie, SkyTrieMut};

mod edit;
mod edit_tests;
mod edit_walks;
mod mem_insert;

#[tokio::test]
async fn empty_storage_max_id_is_nil() {
    let storage = SkyTrieMut::new();
    assert_eq!(BufferIndex::NIL, storage.max_index());
    assert_eq!(BufferIndex::ZERO, storage.next_index());
}

#[tokio::test]
async fn base_id_zero_is_the_empty_base() {
    let storage = SkyTrieMut::new();
    assert_eq!(Base::empty(), storage.get_base(BufferIndex::ZERO, 1));
}

#[tokio::test]
async fn empty_storage_root_is_empty() {
    let storage = SkyTrieMut::new();
    assert_eq!(MapBase::empty(), storage.get_root());
}

#[tokio::test]
async fn root_round_trip_works() {
    let mut edit = SkyTrie::new().into_edit().await;
    let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut edit).await;
    edit.push_root(root).await;
    assert_eq!(root, edit.get_root());
    let view = edit.commit().await;
    assert_eq!(root, view.get_root());
}

#[tokio::test]
async fn append_assigns_sequential_ids() {
    let mut storage = SkyTrieMut::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), &mut storage).await;
    let id0 = storage.push_base(base.clone()).await;
    let id1 = storage.push_base(base.clone()).await;
    assert_eq!(BufferIndex(0), id0);
    assert_eq!(BufferIndex(1), id1);
    assert_eq!(BufferIndex(1), storage.max_index());
    assert_eq!(BufferIndex(2), storage.next_index());
    assert_eq!(base, storage.get_base(id0, base.slots.len()));
    assert_eq!(base, storage.get_base(id1, base.slots.len()));
}

#[tokio::test]
async fn mem_readonly_snapshot_does_not_see_new_bases() {
    let mut load = SkyTrie::new();
    let (base, id) = load
        .edit(async |storage| {
            let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), storage).await;
            let id = storage.push_base(base.clone()).await;
            Ok((base, id))
        })
        .await
        .unwrap();
    let view = load.snapshot();
    let max_id = load
        .edit(async |storage| {
            storage.push_base(base.clone()).await;
            let max_id = storage.max_index();
            Ok(max_id)
        })
        .await
        .unwrap();
    assert_eq!(BufferIndex(1), max_id);
    assert_eq!(id, view.max_index());
    assert_eq!(base, view.get_base(id, base.slots.len()));
}

#[tokio::test]
async fn reading_beyond_max_id_produces_empty() {
    let mut load = SkyTrie::new();
    let base = load
        .edit(async |storage| {
            let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), storage).await;
            storage.push_base(base.clone()).await;
            Ok(base)
        })
        .await
        .unwrap();
    let view = load.snapshot();
    let new_id = load
        .edit(async |storage| {
            let new_id = storage.push_base(base.clone()).await;
            Ok(new_id)
        })
        .await
        .unwrap();
    let get_base = view.get_base(new_id, base.slots.len());
    assert_eq!(get_base, Base::empty())
}

#[tokio::test]
async fn reading_unwritten_id_produces_empty() {
    let storage = SkyTrie::new().snapshot();
    let base = storage.get_base(BufferIndex(1), 0);
    assert_eq!(base, Base::empty())
}
