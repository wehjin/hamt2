use sky_trie::{Insert, InsertCursor, QueryCursor, SkyTrie, TrieValue};

#[tokio::test]
async fn into_iter_yields_u32s() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(1, TrieValue::U32(1)).await;
        trie.insert(2, TrieValue::U32(2)).await;
        trie.insert_deep([3, 4], TrieValue::U32(34), false).await;
        Ok(())
    })
    .await
    .unwrap();

    let mut u32s = trie.into_iter().u32s().collect::<Vec<_>>();
    u32s.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(1, 1), (2, 2)], u32s);
}

#[tokio::test]
async fn into_iter_yields_map_bases() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(1, TrieValue::U32(1)).await;
        trie.insert(2, TrieValue::U32(2)).await;
        trie.insert_deep([3, 4], TrieValue::U32(34), false).await;
        Ok(())
    })
    .await
    .unwrap();

    let map_bases = trie.into_iter().map_bases().collect::<Vec<_>>();
    assert_eq!(1, map_bases.len());
    assert_eq!(3, map_bases[0].0);
}

#[tokio::test]
async fn into_iter_empty() {
    let trie = SkyTrie::new();
    assert!(trie.into_iter().collect::<Vec<_>>().is_empty());
}

#[tokio::test]
async fn into_iter_single() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(7, TrieValue::U32(7)).await;
        Ok(())
    })
    .await
    .unwrap();

    assert_eq!(vec![(7, TrieValue::U32(7))], trie.into_iter().collect::<Vec<_>>());
}

#[tokio::test]
async fn into_iter_many() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        for key in 0..35 {
            trie.insert(key, TrieValue::U32(key as u32)).await;
        }
        Ok(())
    })
    .await
    .unwrap();

    let mut kvs = trie.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);
    let expected = (0..35)
        .map(|key| (key, TrieValue::U32(key as u32)))
        .collect::<Vec<_>>();
    assert_eq!(expected, kvs);
}

#[tokio::test]
async fn into_iter_descended() {
    let mut trie = SkyTrie::new();
    trie.edit(async |trie| {
        trie.insert(1, TrieValue::U32(1)).await;
        trie.insert(2, TrieValue::U32(2)).await;
        trie.insert_deep([3, 4], TrieValue::U32(34), false).await;
        trie.insert_deep([3, 5], TrieValue::U32(35), false).await;
        Ok(())
    })
    .await
    .unwrap();

    let mut sub = trie.clone();
    sub.descend(3);

    let mut kvs = sub.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(4, TrieValue::U32(34)), (5, TrieValue::U32(35))], kvs);
}
