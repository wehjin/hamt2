use crate::storage::MemTrie;
use crate::trie::TrieQuery;

#[tokio::test]
async fn mem_trie_queries() {
    let m = MemTrie::new();
    let v = m.query_all().await.unwrap();
    assert_eq!(v.len(), 0);
}
