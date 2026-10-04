use sky_trie::{Insert, InsertCursor, SkyTrie, TrieValue};

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
