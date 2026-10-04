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

    let mut kvs = trie.into_iter().collect::<Vec<_>>();
    kvs.sort_by_key(|(key, _)| *key);

    // Top level has keys 1, 2 (u32) and 3 (a sub-trie).
    assert_eq!(3, kvs.len());
    assert!(matches!(&kvs[2], (3, TrieValue::SubTrie(_))));

    let mut u32s = kvs
        .iter()
        .filter_map(|(key, value)| match value {
            TrieValue::U32(value) => Some((*key, *value)),
            _ => None,
        })
        .collect::<Vec<_>>();
    u32s.sort_by_key(|(key, _)| *key);
    assert_eq!(vec![(1, 1), (2, 2)], u32s);
}
