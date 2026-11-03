use sky_trie::{Insert, InsertCursor, QueryCursor, SkyTrie, TrieValue};

#[test]
fn into_iter_yields_u32s() {
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        trie.insert(1, TrieValue::U32(1));
        trie.insert(2, TrieValue::U32(2));
        trie.insert_deep([3, 4], TrieValue::U32(34), false);
        Ok(())
    })
    .unwrap();

    let mut u32s = trie.into_iter().u32s().collect::<Vec<_>>();
    u32s.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(1, 1), (2, 2)], u32s);
}

#[test]
fn into_iter_yields_map_bases() {
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        trie.insert(1, TrieValue::U32(1));
        trie.insert(2, TrieValue::U32(2));
        trie.insert_deep([3, 4], TrieValue::U32(34), false);
        Ok(())
    })
    .unwrap();

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
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        trie.insert(7, TrieValue::U32(7));
        Ok(())
    })
    .unwrap();

    assert_eq!(
        vec![(7, TrieValue::U32(7))],
        trie.into_iter().collect::<Vec<_>>()
    );
}

#[test]
fn into_iter_many() {
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        for key in 0..35 {
            trie.insert(key, TrieValue::U32(key as u32));
        }
        Ok(())
    })
    .unwrap();

    let mut kvs = trie.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);
    let expected = (0..35)
        .map(|key| (key, TrieValue::U32(key as u32)))
        .collect::<Vec<_>>();
    assert_eq!(expected, kvs);
}

#[test]
fn into_iter_descended() {
    let mut trie = SkyTrie::new();
    trie.edit(|trie| {
        trie.insert(1, TrieValue::U32(1));
        trie.insert(2, TrieValue::U32(2));
        trie.insert_deep([3, 4], TrieValue::U32(34), false);
        trie.insert_deep([3, 5], TrieValue::U32(35), false);
        Ok(())
    })
    .unwrap();

    let mut sub = trie.clone();
    sub.descend(3);

    let mut kvs = sub.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(4, TrieValue::U32(34)), (5, TrieValue::U32(35))], kvs);
}
