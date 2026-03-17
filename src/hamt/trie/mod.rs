pub mod core;
pub mod key;
pub mod mem;
pub mod space;

#[cfg(test)]
mod tests {
	use crate::hamt::trie::space::SpaceTrie;

	#[tokio::test]
    async fn later_insertion_overwrites_earlier_insertion() {
        let trie = SpaceTrie::new()
            .insert(-1, 42)
            .unwrap()
            .insert(-1, 43)
            .unwrap();
        let value = trie.query_value(-1).unwrap();
        assert_eq!(Some(43), value);
    }
}
