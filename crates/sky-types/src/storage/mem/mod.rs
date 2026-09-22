use crate::storage::{
    ReadStorage, ReadStorageError, ReadWriteStorage, StorageStatus, WriteStorageError,
};
use crate::trie::{MapBase, Base, BaseId, TrieRead};
use std::sync::{Arc, RwLock};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that
/// [`BaseId::ZERO`] always reads back the empty base and no storage is
/// wasted storing it.
#[derive(Debug)]
pub struct MemStorage {
    inner: MemReadStorage,
}

impl TrieRead for MemStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
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

/// A read-only snapshot of a [`MemStorage`], taken at
/// [`ReadStorage::snapshot`] time. `max_id` and `root` are captured
/// when the snapshot is created, so later appends to the writer are invisible
/// through it; the base pool itself is shared read-only.
#[derive(Debug, Clone)]
pub struct MemReadStorage {
    bases: Arc<RwLock<Vec<Base>>>,
    status: StorageStatus,
}

impl TrieRead for MemReadStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        assert!(id <= self.max_id(), "id out of bounds {id:?}");
        let index = id.0 as usize;
        let read = self.bases.read().unwrap();
        let base = read[index].clone();
        Ok(base)
    }

    fn read_root(&self) -> MapBase {
        self.status().root
    }
}

impl ReadStorage for MemReadStorage {
    type Snapshot = MemReadStorage;

    fn status(&self) -> StorageStatus {
        self.status
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let status = self.status.with_new_root(new_root);
        Self { status, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl MemReadStorage {
    pub fn empty() -> Self {
        let bases = Arc::new(RwLock::new(vec![Base::empty()]));
        let status = StorageStatus::default();
        Self { bases, status }
    }
}
