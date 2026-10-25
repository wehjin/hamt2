use crate::Insert;
use crate::services::map_base::one_kv;
use crate::{Base, Buffer, BufferIndex, BufferMut, HashKey, MapBase, TrieValue};
use crate::{SkyTrie, SkyTrieMut};
use crate::{Slot, Snap};

mod edit;
mod edit_tests;
mod edit_walks;
mod mem_insert;

#[tokio::test]
async fn base_id_zero_is_the_seed_base() {
    let trie = SkyTrieMut::new();
    assert_eq!(Base::seed().slots, trie.get_base(BufferIndex::ZERO, 1));
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
async fn append_assigns_distinct_ids() {
    let mut trie = SkyTrieMut::new();
    let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7), &mut trie).await;
    let id0 = trie.push_base(base.clone()).await;
    let id1 = trie.push_base(base.clone()).await;
    assert_ne!(id0, id1);
    assert_eq!(base.slots, trie.get_base(id0, base.slots.len()));
    assert_eq!(base.slots, trie.get_base(id1, base.slots.len()));
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
    let root_before = view.get_root();
    trie.edit(async |trie| {
        trie.insert(8, 8).await;
        Ok(())
    })
    .await
    .unwrap();
    assert_ne!(view.get_root(), trie.get_root());
    assert_eq!(view.get_root(), root_before);
}

#[tokio::test]
async fn reading_unwritten_id_produces_empty() {
    let trie = SkyTrie::new().snapshot();
    let base = trie.get_base(BufferIndex(1), 0);
    assert_eq!(base, &[] as &[Slot])
}
