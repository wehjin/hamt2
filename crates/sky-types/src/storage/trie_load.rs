use crate::storage::traits::BaseStore;
use crate::storage::trie_edit::TrieEdit;
use crate::storage::{Mem, ReadStorageError, TrieView};
use crate::trie::BaseView;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};

pub fn mem_load_new() -> TrieLoad<Mem> {
    TrieLoad::load(Mem::new())
}

#[derive(Debug)]
pub struct TrieLoad<S: BaseStore> {
    inner: TrieView<S>,
}

impl<S: BaseStore + Send + Sync> TrieLoad<S> {
    pub fn load(store: S) -> Self {
        let inner = TrieView::load(store);
        Self { inner }
    }

    pub async fn edit<F, Out>(&mut self, f: F) -> anyhow::Result<Out>
    where
        F: AsyncFnOnce(&mut TrieEdit<S>) -> anyhow::Result<Out>,
    {
        let mut edit = TrieEdit::extend(&self.inner).await?;
        let out = f(&mut edit).await?;
        let committed = edit.commit().await?;
        self.inner = committed.snapshot();
        Ok(out)
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for TrieLoad<S> {
    type Subtrie = TrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl<S: BaseStore + Send + Sync> BaseView for TrieLoad<S> {
    type Snapshot = TrieView<S>;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl<S: BaseStore> BaseRead for TrieLoad<S> {
    fn max_id(&self) -> BaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}
