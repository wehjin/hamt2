use crate::{SkyTrie, TrieValue};
use std::collections::HashMap;

#[test]
fn max_u32_value_insertions_works() {
    let trie = SkyTrie::new().insert(100, u32::MAX);
    assert_eq!(trie.query_u32(100), Some(u32::MAX));
}

#[test]
#[should_panic(expected = "assertion failed: value >= 0")]
fn negative_key_insertions_panic() {
    let trie = SkyTrie::new();
    let _ = trie.insert(-1, 100);
}

#[test]
fn multiple_insertions_work() {
    let trie = SkyTrie::new().insert(1, 1);
    let values = trie
        .clone()
        .into_iter()
        .u32s()
        .collect::<HashMap<i32, u32>>();
    assert_eq!(values.get(&1), Some(&1));
    let trie = trie.insert(2, 2);
    assert_eq!(TrieValue::U32(1), trie.query(1).unwrap());
    assert_eq!(TrieValue::U32(2), trie.query(2).unwrap());
    let values = trie.into_iter().u32s().collect::<HashMap<i32, u32>>();
    assert_eq!(values.get(&1), Some(&1));
    assert_eq!(values.get(&2), Some(&2));
}

#[test]
fn snapshot_queries_work() {
    let trie = SkyTrie::new()
        .insert(1, TrieValue::U32(42))
        .insert_deep([2, 42], TrieValue::U32(242));

    let snap = trie.clone();
    assert_eq!(Some(42), snap.query_u32(1));
    assert_eq!(Some(TrieValue::U32(242)), snap.query_deep([2, 42]));
}

#[test]
fn multi_depth_saturation_in_multiple_edits_work() {
    // Insert values past root saturation.
    let mut trie = SkyTrie::new().insert(100, TrieValue::U32(42));
    for a in 0..=32 {
        trie = trie.insert_deep([3, a], TrieValue::U32(a as u32));
    }

    // Check insertions are present.
    assert_eq!(Some(42), trie.query_u32(100));
    for a in 0..=32 {
        assert_eq!(Some(TrieValue::U32(a as u32)), trie.query_deep([3, a]));
    }

    // Deep insert values past root-block saturation at multiple sub-trie depths.
    for i in 0..35 {
        let e = 5 + i;
        trie = trie.insert_deep([e, 0], TrieValue::U32(e as u32));
    }
    for i in 0..35 {
        let a = 3 + i;
        trie = trie.insert_deep([4, a], TrieValue::U32(a as u32));
    }
    for a in 32..=64 {
        trie = trie.insert_deep([3, a], TrieValue::U32(a as u32));
    }

    // Test post-commit insertions.
    for a in 0..=64 {
        assert_eq!(Some(TrieValue::U32(a as u32)), trie.query_deep([3, a]));
    }
}

#[test]
fn later_insertion_overwrites_earlier_insertion() {
    let trie = SkyTrie::new().insert(1, 42).insert(1, TrieValue::U32(43));
    assert_eq!(Some(43), trie.query_u32(1));
}

#[test]
fn different_keys_store_different_values() {
    // 33 keys will saturate the root block.
    let keys = (0..=32).collect::<Vec<_>>();
    let mut trie = SkyTrie::new();
    for i in &keys {
        trie = trie.insert(*i, *i as u32);
    }
    let values = trie.into_iter().u32s().collect::<HashMap<i32, u32>>();
    let expected = keys
        .iter()
        .map(|k| (*k, *k as u32))
        .collect::<HashMap<i32, u32>>();
    assert_eq!(values, expected);
}

#[test]
fn deep_insertions_work_basic() {
    let trie = SkyTrie::new().insert_deep([2, 42], 242);
    assert_eq!(Some(TrieValue::U32(242)), trie.query_deep([2, 42]));
}

#[test]
fn deep_insertions_work_with_saturated_root() {
    let mut trie = SkyTrie::new();
    for e in 0..=33 {
        trie = trie.insert_deep([e, e], TrieValue::U32(e as u32));
    }
    {
        let value = trie.query_deep([4]);
        let Some(TrieValue::Trie(sky_trie)) = value else {
            panic!("expected trie");
        };
        assert_eq!(1, sky_trie.root.map.slot_count());
    }
    {
        let value = trie.query_deep([4, 4]);
        assert_eq!(Some(TrieValue::U32(4)), value);
    }
    {
        let value = trie.query_deep([4, 1]);
        assert_eq!(None, value);
    }
    {
        let value = trie.query_deep([5, 2]);
        assert_eq!(None, value);
    }
}

#[test]
#[should_panic(expected = "assertion failed: value >= 0")]
fn deep_queries_panic_for_invalid_key() {
    let trie = SkyTrie::new();
    trie.query_deep([4, 4, -1]).unwrap();
}
