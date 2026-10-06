use crate::services::map_base::one_kv;
use crate::{Base, Buffer, BufferIndex, BufferMut, HashKey, MapBase, TrieValue};
use crate::{Insert, Query, Snap};
use crate::{SkyTrie, SkyTrieMut};

mod edit;
mod edit_tests;
mod edit_walks;
mod mem_insert;

#[tokio::test]
async fn empty_trie_max_id_is_zero() {
    let trie = SkyTrieMut::new();
    assert_eq!(BufferIndex::ZERO, trie.max_index());
    assert_eq!(BufferIndex::ONE, trie.next_index());
}

#[tokio::test]
async fn base_id_zero_is_the_seed_base() {
    let trie = SkyTrieMut::new();
    assert_eq!(Base::seed(), trie.get_base(BufferIndex::ZERO, 1));
}

#[tokio::test]
async fn empty_trie_root_is_empty() {
    let trie = SkyTrieMut::new();
    assert_eq!(MapBase::empty(), trie.get_root());
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
    let mut trie = SkyTrieMut::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), &mut trie).await;
    let id0 = trie.push_base(base.clone()).await;
    let id1 = trie.push_base(base.clone()).await;
    assert_eq!(BufferIndex(1), id0);
    assert_eq!(BufferIndex(2), id1);
    assert_eq!(BufferIndex(2), trie.max_index());
    assert_eq!(BufferIndex(3), trie.next_index());
    assert_eq!(base, trie.get_base(id0, base.slots.len()));
    assert_eq!(base, trie.get_base(id1, base.slots.len()));
}

#[tokio::test]
async fn mem_readonly_snapshot_does_not_see_new_bases() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(7, 7).await;
        Ok(())
    })
    .await
    .unwrap();
    let index1 = trie.max_index();
    let view = trie.snapshot();
    trie.edit(async |trie| {
        trie.insert(8, 8).await;
        Ok(())
    })
    .await
    .unwrap();
    assert_ne!(view.max_index(), trie.max_index());
    assert_eq!(view.max_index(), index1);
}

#[tokio::test]
async fn reading_beyond_max_id_produces_empty() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(7, 7).await;
        Ok(())
    })
    .await
    .unwrap();
    let view = trie.snapshot();
    trie.edit(async |trie| {
        trie.insert(8, 8).await;
        Ok(())
    })
    .await
    .unwrap();
    assert_ne!(view.max_index(), trie.max_index());
    assert_eq!(view.get_base(trie.max_index(), 1), Base::empty());
}

#[tokio::test]
async fn reading_unwritten_id_produces_empty() {
    let trie = SkyTrie::new().snapshot();
    let base = trie.get_base(BufferIndex(1), 0);
    assert_eq!(base, Base::empty())
}
