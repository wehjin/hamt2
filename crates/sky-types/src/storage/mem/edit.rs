use crate::storage::{
    MemTrieView, ReadStorageError, StorageStatus, TrieEdit, TrieView, WriteStorageError,
};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase, TrieStream};

#[derive(Debug)]
pub struct MemTrieEdit {
    pub(crate) inner: MemTrieView,
}

impl MemTrieEdit {
    /// Make this pub(crate) after moving usage out of local.
    pub fn new() -> Self {
        let inner = MemTrieView::empty();
        Self { inner }
    }

    pub(crate) fn extend(past: &MemTrieView) -> Self {
        let inner = MemTrieView::extend(past);
        Self { inner }
    }
}

impl TrieEdit for MemTrieEdit {
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }
}

impl BaseCommit for MemTrieEdit {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.inner.root = root;
        Ok(())
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let id = self.next_id();
        {
            let mut bases = self.inner.bases.write().await;
            bases.push(base);
        }
        self.inner.max_id = id;
        Ok(id)
    }
}

impl TrieStream for MemTrieEdit {
    type Subtrie = MemTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl TrieView for MemTrieEdit {
    type Snapshot = MemTrieView;

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

impl BaseRead for MemTrieEdit {
    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::MemTrieEdit;
    use crate::trie::{TrieInsert, TrieQuery, TrieStream, TrieValue};
    use futures::StreamExt;

    #[tokio::test]
    async fn query_value_exists_works() {
        let mut m = MemTrieEdit::new();
        m.insert(32, TrieValue::U32(33)).await.unwrap();
        let v = m.query(32).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(33)));
    }

    #[tokio::test]
    async fn u32_stream_exists_works() {
        let mut m = MemTrieEdit::new();
        m.insert(27, TrieValue::U32(28)).await.unwrap();
        let v = m.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(v, vec![(27, 28)]);
    }

    #[tokio::test]
    async fn deep_insert_and_query_works() {
        let mut m = MemTrieEdit::new();
        m.deep_insert([1, 2, 3], 45, false).await.unwrap();
        let v = m.deep_query([1, 2, 3]).await.unwrap();
        assert_eq!(v, Some(TrieValue::U32(45)));
    }
}
