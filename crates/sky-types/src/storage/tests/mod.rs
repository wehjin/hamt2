use crate::storage::load::StoreLoad;
use crate::storage::{MemEdit, MemLoad};
use crate::trie::TrieSnap;
use crate::trie::map_base::one_kv;
use crate::trie::{Base, BufferMut, BufferIndex, Buffer, HashKey, MapBase, TrieValue};

mod edit;
mod mem_insert;
mod mem_stream;

#[tokio::test]
async fn empty_storage_max_id_is_zero() {
    let storage = MemEdit::new();
    assert_eq!(BufferIndex::ZERO, storage.max_index());
    assert_eq!(BufferIndex(1), storage.next_index());
}

#[tokio::test]
async fn base_id_zero_is_the_empty_base() {
    let storage = MemEdit::new();
    assert_eq!(
        Base::empty(),
        storage.read_base(BufferIndex::ZERO).await.expect("read")
    );
}

#[tokio::test]
async fn empty_storage_root_is_empty() {
    let storage = MemEdit::new();
    assert_eq!(MapBase::empty(), storage.read_root());
}

#[tokio::test]
async fn root_round_trip_works() {
    let mut load = MemLoad::new();
    let root = {
        let mut storage = load.begin_edit().await.unwrap();
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
            .await
            .expect("root");
        storage.commit_root(root).await.expect("write root");
        assert_eq!(root, storage.read_root());
        load.commit_edit(storage).await.unwrap();
        root
    };
    let view = load.snapshot();
    assert_eq!(root, view.read_root());
}

#[tokio::test]
async fn append_assigns_sequential_ids() {
    let mut storage = MemEdit::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    let id0 = storage.commit_base(base.clone()).await.expect("append");
    let id1 = storage.commit_base(base.clone()).await.expect("append");
    assert_eq!(BufferIndex(1), id0);
    assert_eq!(BufferIndex(2), id1);
    assert_eq!(BufferIndex(2), storage.max_index());
    assert_eq!(BufferIndex(3), storage.next_index());
    assert_eq!(base, storage.read_base(id0).await.expect("read"));
    assert_eq!(base, storage.read_base(id1).await.expect("read"));
}

#[tokio::test]
async fn mem_readonly_snapshot_does_not_see_new_bases() {
    let mut load = MemLoad::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    let id = load
        .edit(async |storage| {
            let id = storage.commit_base(base.clone()).await.expect("append");
            Ok(id)
        })
        .await
        .unwrap();
    let view = load.snapshot();
    let max_id = load
        .edit(async |storage| {
            storage.commit_base(base.clone()).await.expect("append");
            let max_id = storage.max_index();
            Ok(max_id)
        })
        .await
        .unwrap();
    assert_eq!(BufferIndex(2), max_id);
    assert_eq!(id, view.max_index());
    assert_eq!(base, view.read_base(id).await.expect("read"));
}

#[tokio::test]
async fn reading_beyond_max_id_produces_empty() {
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    let mut load = MemLoad::new();
    load.edit(async |storage| {
        storage.commit_base(base.clone()).await.expect("append");
        Ok(())
    })
    .await
    .unwrap();
    let view = load.snapshot();
    let new_id = load
        .edit(async |storage| {
            let new_id = storage.commit_base(base.clone()).await?;
            Ok(new_id)
        })
        .await
        .unwrap();
    let base = view.read_base(new_id).await.expect("read");
    assert_eq!(base, Base::empty())
}

#[tokio::test]
async fn reading_unwritten_id_produces_empty() {
    let storage = MemLoad::new().snapshot();
    let base = storage.read_base(BufferIndex(1)).await.expect("read");
    assert_eq!(base, Base::empty())
}
