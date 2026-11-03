use sky_trie::SkyTrie;

#[test]
fn list_keys_works() {
    let trie = SkyTrie::new().insert(33, 33).insert(34, 34);

    let mut keys = trie.list_keys();
    keys.sort();
    assert_eq!(keys, vec![33, 34])
}
