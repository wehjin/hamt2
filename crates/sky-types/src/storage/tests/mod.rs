use crate::storage::mem::MemStorage;
use crate::storage::{ReadStorage, ReadWriteStorage};
use crate::trie::map_base::one_kv;
use crate::trie::{HashKey, MapBase, Base, BaseId, RootBaseRead, TrieValue};

#[tokio::test]
async fn empty_storage_max_id_is_zero() {
    let storage = MemStorage::new();
    assert_eq!(BaseId::ZERO, storage.max_id());
    assert_eq!(BaseId(1), storage.next_id());
}

#[tokio::test]
async fn base_id_zero_is_the_empty_base() {
    let storage = MemStorage::new();
    assert_eq!(
	    Base::empty(),
	    storage.read_base(BaseId::ZERO).await.expect("read")
    );
}

#[tokio::test]
async fn empty_storage_root_is_empty() {
    let storage = MemStorage::new();
    assert_eq!(MapBase::empty(), storage.read_root());
}

#[tokio::test]
async fn root_round_trip_works() {
    let mut storage = MemStorage::new();
    let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
        .await
        .expect("root");
    storage.write_root(root).await.expect("write root");
    let view = storage.snapshot();
    assert_eq!(root, storage.read_root());
    assert_eq!(root, view.read_root());
}

#[tokio::test]
async fn append_assigns_sequential_ids() {
    let mut storage = MemStorage::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    let id0 = storage.append(&base).await.expect("append");
    let id1 = storage.append(&base).await.expect("append");
    assert_eq!(BaseId(1), id0);
    assert_eq!(BaseId(2), id1);
    assert_eq!(BaseId(2), storage.max_id());
    assert_eq!(BaseId(3), storage.next_id());
    assert_eq!(base, storage.read_base(id0).await.expect("read"));
    assert_eq!(base, storage.read_base(id1).await.expect("read"));
}

#[tokio::test]
async fn mem_readonly_snapshot_does_not_see_new_bases() {
    let mut storage = MemStorage::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    let id = storage.append(&base).await.expect("append");
    let view = storage.snapshot();
    storage.append(&base).await.expect("append");
    assert_eq!(BaseId(2), storage.max_id());
    assert_eq!(id, view.max_id());
    assert_eq!(base, view.read_base(id).await.expect("read"));
}

#[tokio::test]
#[should_panic(expected = "id out of bounds")]
async fn mem_snapshot_panics_reading_beyond_max_id() {
    let mut storage = MemStorage::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
    storage.append(&base).await.expect("append");
    let view = storage.snapshot();
    let new_id = storage.append(&base).await.expect("append");
    let _ = view.read_base(new_id).await;
}

#[tokio::test]
#[should_panic(expected = "id out of bounds")]
async fn read_panics_on_unwritten_id() {
    let storage = MemStorage::new();
    let _ = storage.read_base(BaseId(1)).await;
}
