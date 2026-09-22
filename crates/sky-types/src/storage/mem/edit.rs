use crate::storage::{
    MemTrieView, ReadStorage, ReadStorageError, ReadWriteStorage, StorageStatus, WriteStorageError,
};
use crate::trie::{Base, BaseId, MapBase, RootBaseRead, TrieStream};

#[derive(Debug)]
pub struct MemTrieEdit {
    inner: MemTrieView,
}

impl MemTrieEdit {
    pub fn new() -> Self {
        let inner = MemTrieView::empty();
        Self { inner }
    }
}

impl TrieStream for MemTrieEdit {
    type Subtrie = MemTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.inner.to_subtrie(subtrie_root)
    }
}

impl ReadWriteStorage for MemTrieEdit {
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }

    async fn append(&mut self, base: &Base) -> Result<BaseId, WriteStorageError> {
        let id = self.next_id();
        {
            let mut bases = self.inner.bases.write().unwrap();
            bases.push(base.clone());
        }
        self.inner.status.max_id = id;
        Ok(id)
    }

    async fn write_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.inner.status.root = root;
        Ok(())
    }
}

impl ReadStorage for MemTrieEdit {
    type Snapshot = MemTrieView;

    fn status(&self) -> StorageStatus {
        self.inner.status()
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl RootBaseRead for MemTrieEdit {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::{MemTrieEdit, ReadWriteStorage};
    use crate::trie::{HashKey, TrieStream, TrieValue, map_base};
    use futures::StreamExt;

    #[tokio::test]
    async fn mem_trie_mut_works() {
        let mut m = MemTrieEdit::new();
        let map_base = map_base::one_kv(HashKey::new(27), TrieValue::U32(28), &mut m)
            .await
            .unwrap();
        m.write_root(map_base).await.unwrap();
        let values = m.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(values, vec![(27, 28)]);
    }
}
