use crate::storage::{
    BaseStore, MemBaseStore, MemTrieCore, MemTrieView, ReadStorageError, StorageStatus, TrieEdit,
    TrieView, WriteStorageError,
};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase, TrieStream};

pub fn mem_edit_new() -> MemTrieCore<MemBaseStore> {
    let store = MemBaseStore::new();
    let edit = MemTrieCore::load(store);
    edit
}

#[derive(Debug)]
pub struct MemTrieEdit<S: BaseStore> {
    pub(crate) inner: MemTrieCore<S>,
}

impl<S: BaseStore> MemTrieEdit<S> {
    pub fn load(store: S) -> Self {
        let inner = MemTrieCore::load(store);
        Self { inner }
    }

    pub(crate) async fn extend(past: &MemTrieView<S>) -> Result<Self, WriteStorageError> {
        let inner = MemTrieCore::extend(past).await?;
        let extension = Self { inner };
        Ok(extension)
    }
}

impl<S: BaseStore + Send + Sync> TrieEdit for MemTrieEdit<S> {
    fn next_id(&self) -> BaseId {
        self.inner.next_id()
    }
}

impl<S: BaseStore> BaseCommit for MemTrieEdit<S> {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.inner.commit_root(root).await
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        self.inner.commit_base(base).await
    }
}

impl<S: BaseStore + Send + Sync> TrieStream for MemTrieEdit<S> {
    type Subtrie = MemTrieView<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl<S: BaseStore + Send + Sync> TrieView for MemTrieEdit<S> {
    type Snapshot = MemTrieView<S>;

    fn status(&self) -> StorageStatus {
        self.inner.status()
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl<S: BaseStore> BaseRead for MemTrieEdit<S> {
    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::mem_edit_new;
    use crate::trie::{TrieInsert, TrieQuery, TrieStream, TrieValue};
    use futures::StreamExt;

    #[tokio::test]
    async fn query_value_exists_works() {
        let mut m = mem_edit_new();
        m.insert(32, TrieValue::U32(33)).await.unwrap();
        let v = m.query(32).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(33)));
    }

    #[tokio::test]
    async fn u32_stream_exists_works() {
        let mut m = mem_edit_new();
        m.insert(27, TrieValue::U32(28)).await.unwrap();
        let v = m.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(v, vec![(27, 28)]);
    }

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut m = mem_edit_new();
        m.deep_insert([1, 2, 3], 45, false).await.unwrap();
        let v = m.deep_query([1, 2, 3]).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(45)));
    }
}
