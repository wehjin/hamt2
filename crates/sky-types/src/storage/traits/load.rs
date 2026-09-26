use crate::storage::edit::StoreEdit;
use crate::trie::TrieWalk;
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

#[allow(async_fn_in_trait)]
pub trait StoreLoad: TrieWalk + TrieStream + TrieSnap + TrieQuery + Send {
    type Edit: StoreEdit;

    fn new() -> Self;

    async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut Self::Edit) -> anyhow::Result<Out>;
}
