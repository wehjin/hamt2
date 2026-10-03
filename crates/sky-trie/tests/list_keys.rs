use sky_trie::{Insert, SkyTrie};

#[tokio::test]
async fn list_keys_works() {
    let mut trie = SkyTrie::new();
    trie.edit(async |edit| {
        edit.insert(33, 33).await;
        edit.insert(34, 34).await;
        Ok(())
    })
    .await
    .unwrap();

    let mut keys = trie.list_keys().await;
    keys.sort();
    assert_eq!(keys, vec![33, 34])
}
