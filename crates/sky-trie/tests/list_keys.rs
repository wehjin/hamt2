use sky_trie::{Insert, SkyTrie};

#[test]
fn list_keys_works() {
    let mut trie = SkyTrie::new();
    trie.edit(|edit| {
        edit.insert(33, 33);
        edit.insert(34, 34);
        Ok(())
    })
    .unwrap();

    let mut keys = trie.list_keys();
    keys.sort();
    assert_eq!(keys, vec![33, 34])
}
