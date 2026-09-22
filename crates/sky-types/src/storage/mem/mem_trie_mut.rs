use crate::storage::{
    MemReadStorage, ReadStorage, ReadStorageError, ReadWriteStorage, StorageStatus,
    WriteStorageError,
};
use crate::trie::{Base, BaseId, MapBase, RootBaseRead};

#[derive(Debug)]
pub struct MemStorage {
    inner: MemReadStorage,
}

impl MemStorage {
    pub fn new() -> Self {
        let inner = MemReadStorage::empty();
        Self { inner }
    }
}

impl ReadWriteStorage for MemStorage {
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

impl ReadStorage for MemStorage {
    type Snapshot = MemReadStorage;

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

impl RootBaseRead for MemStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}
