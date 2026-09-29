use crate::trie::{MapBase, TrieInsertError, TrieKey};

#[allow(async_fn_in_trait)]
pub trait QueryCursor {
    fn top_root(&self) -> MapBase;
    fn ascend(&mut self) -> Option<(TrieKey, MapBase)>;
}

#[allow(async_fn_in_trait)]
pub trait InsertCursor {
    async fn descend_insert(&mut self, key: impl Into<TrieKey>) -> Result<(), TrieInsertError>;
}
