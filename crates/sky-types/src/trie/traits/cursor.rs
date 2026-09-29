use crate::trie::{MapBase, TrieKey, TrieQueryError};

#[allow(async_fn_in_trait)]
pub trait QueryCursor {
    fn top_root(&self) -> MapBase;
    fn ascend(&mut self) -> Option<(TrieKey, MapBase)>;
    async fn descend(&mut self, key: impl Into<TrieKey>) -> Result<(), TrieQueryError>;
}

#[allow(async_fn_in_trait)]
pub trait InsertCursor {}
