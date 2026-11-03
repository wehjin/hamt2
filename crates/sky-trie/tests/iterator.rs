use sky_trie::{SkyTrie, TrieValue};

#[test]
fn into_iter_yields_u32s() {
    let trie = SkyTrie::new()
        .insert(1, TrieValue::U32(1))
        .insert(2, TrieValue::U32(2))
        .insert_deep([3, 4], TrieValue::U32(34));

    let mut u32s = trie.into_iter().u32s().collect::<Vec<_>>();
    u32s.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(1, 1), (2, 2)], u32s);
}

#[test]
fn into_iter_yields_map_bases() {
    let trie = SkyTrie::new()
        .insert(1, TrieValue::U32(1))
        .insert(2, TrieValue::U32(2))
        .insert_deep([3, 4], TrieValue::U32(34));

    let map_bases = trie.into_iter().map_bases().collect::<Vec<_>>();
    assert_eq!(1, map_bases.len());
    assert_eq!(3, map_bases[0].0);
}

#[test]
fn into_iter_empty() {
    let trie = SkyTrie::new();
    assert!(trie.into_iter().collect::<Vec<_>>().is_empty());
}

#[test]
fn into_iter_single() {
    let trie = SkyTrie::new().insert(7, TrieValue::U32(7));
    assert_eq!(
        vec![(7, TrieValue::U32(7))],
        trie.into_iter().collect::<Vec<_>>()
    );
}

#[test]
fn into_iter_many() {
    let mut trie = SkyTrie::new();
    for key in 0..35 {
        trie = trie.insert(key, TrieValue::U32(key as u32));
    }

    let mut kvs = trie.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);
    let expected = (0..35)
        .map(|key| (key, TrieValue::U32(key as u32)))
        .collect::<Vec<_>>();
    assert_eq!(expected, kvs);
}
