use crate::storage::edit::StoreEdit;
use crate::storage::view::StoreView;
use crate::trie::QueryCursor;
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

#[allow(async_fn_in_trait)]
pub trait StoreLoad:
    QueryCursor + TrieStream + TrieSnap<Snapshot = Self::View> + TrieQuery + Send
{
    type Edit: StoreEdit;
    type View: StoreView;

    fn new() -> Self;

    async fn begin_edit(&self) -> Result<Self::Edit, anyhow::Error>;
    async fn commit_edit(&mut self, edit: Self::Edit) -> Result<(), anyhow::Error>;

    async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut Self::Edit) -> anyhow::Result<Out>,
    {
        let mut edit = self.begin_edit().await?;
        let out = f(&mut edit).await?;
        self.commit_edit(edit).await?;
        Ok(out)
    }
}
