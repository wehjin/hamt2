use crate::storage::load::StoreLoad;
use crate::storage::{MemEdit, MemLoad};
use crate::trie::TrieSnap;
use crate::trie::map_base::one_kv;
use crate::trie::{Base, Buffer, BufferIndex, BufferMut, HashKey, MapBase, TrieValue};

mod edit;
mod mem_insert;
mod mem_stream;

#[tokio::test]
async fn empty_storage_max_id_is_nil() {
    let storage = MemEdit::new();
    assert_eq!(BufferIndex::NIL, storage.max_index());
    assert_eq!(BufferIndex::ZERO, storage.next_index());
}

#[tokio::test]
async fn base_id_zero_is_the_empty_base() {
    let storage = MemEdit::new();
    assert_eq!(Base::empty(), storage.get_base(BufferIndex::ZERO, 1).await);
}

#[tokio::test]
async fn empty_storage_root_is_empty() {
    let storage = MemEdit::new();
    assert_eq!(MapBase::empty(), storage.get_root());
}

#[tokio::test]
async fn root_round_trip_works() {
    let mut load = MemLoad::new();
    let root = {
        let mut storage = load.begin_edit().await.unwrap();
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
            .await
            .expect("root");
        storage.push_root(root).await.expect("write root");
        assert_eq!(root, storage.get_root());
        load.commit_edit(storage).await.unwrap();
        root
    };
    let view = load.snapshot();
    assert_eq!(root, view.get_root());
}

#[tokio::test]
async fn append_assigns_sequential_ids() {
    let mut storage = MemEdit::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
        .await
        .expect("base");
    let id0 = storage.push_base(base.clone()).await.expect("append");
    let id1 = storage.push_base(base.clone()).await.expect("append");
    assert_eq!(BufferIndex(0), id0);
    assert_eq!(BufferIndex(1), id1);
    assert_eq!(BufferIndex(1), storage.max_index());
    assert_eq!(BufferIndex(2), storage.next_index());
    assert_eq!(base, storage.get_base(id0, base.slots.len()).await);
    assert_eq!(base, storage.get_base(id1, base.slots.len()).await);
}

#[tokio::test]
async fn mem_readonly_snapshot_does_not_see_new_bases() {
    let mut load = MemLoad::new();
    let (base, id) = load
        .edit(async |storage| {
            let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), storage).await?;
            let id = storage.push_base(base.clone()).await.expect("append");
            Ok((base, id))
        })
        .await
        .unwrap();
    let view = load.snapshot();
    let max_id = load
        .edit(async |storage| {
            storage.push_base(base.clone()).await.expect("append");
            let max_id = storage.max_index();
            Ok(max_id)
        })
        .await
        .unwrap();
    assert_eq!(BufferIndex(1), max_id);
    assert_eq!(id, view.max_index());
    assert_eq!(base, view.get_base(id, base.slots.len()).await);
}

#[tokio::test]
async fn reading_beyond_max_id_produces_empty() {
    let mut load = MemLoad::new();
    let base = load
        .edit(async |storage| {
            let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), storage).await?;
            storage.push_base(base.clone()).await.expect("append");
            Ok(base)
        })
        .await
        .unwrap();
    let view = load.snapshot();
    let new_id = load
        .edit(async |storage| {
            let new_id = storage.push_base(base.clone()).await?;
            Ok(new_id)
        })
        .await
        .unwrap();
    let get_base = view.get_base(new_id, base.slots.len()).await;
    assert_eq!(get_base, Base::empty())
}

#[tokio::test]
async fn reading_unwritten_id_produces_empty() {
    let storage = MemLoad::new().snapshot();
    let base = storage.get_base(BufferIndex(1), 0).await;
    assert_eq!(base, Base::empty())
}
