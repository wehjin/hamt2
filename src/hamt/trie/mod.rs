pub mod core;
pub mod key;
pub mod mem;
pub mod space;

#[cfg(test)]
mod tests {
	use crate::hamt::trie::key::TrieKey;
	use crate::hamt::trie::space::SpaceTrie;

	#[tokio::test]
    async fn trie_works() {
        let trie = SpaceTrie::new().insert(TrieKey::new(1), 42).unwrap();
        let value = trie.query_value(TrieKey::new(1)).unwrap();
        assert_eq!(Some(42), value);
    }
}
