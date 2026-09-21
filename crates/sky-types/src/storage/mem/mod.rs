use crate::storage::{
    ReadStorage, ReadWriteStorage, StorageHead, StoreRead, VecBases, WriteStorageError,
};
use crate::trie::{MapBase, SlotBase, SlotBaseId};
use std::sync::{Arc, RwLock};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that
/// [`SlotBaseId::ZERO`] always reads back the empty base and no storage is
/// wasted storing it.
#[derive(Debug)]
pub struct MemStorage {
    bases: Arc<RwLock<Vec<SlotBase>>>,
    status: StorageHead,
}

impl VecBases for MemStorage {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase>>> {
        &self.bases
    }
}

impl StoreRead for MemStorage {
    fn status(&self) -> StorageHead {
        self.status
    }
}

impl MemStorage {
    pub fn new() -> Self {
        let bases = Arc::new(RwLock::new(vec![SlotBase::empty()]));
        let status = StorageHead::default();
        Self { bases, status }
    }
}

impl ReadStorage for MemStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        MemReadStorage {
            bases: Arc::clone(&self.bases),
            status: self.status.clone(),
        }
    }
}

impl ReadWriteStorage for MemStorage {
    fn next_id(&self) -> SlotBaseId {
        let inner = self.bases.read().unwrap();
        SlotBaseId(inner.len() as i32)
    }

    async fn append(&mut self, base: &SlotBase) -> Result<SlotBaseId, WriteStorageError> {
        let id = self.next_id();
        let mut inner = self.bases.write().unwrap();
        inner.push(base.clone());
        self.status.max_id = id;
        Ok(id)
    }

    async fn write_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.status.root = root;
        Ok(())
    }
}

/// A read-only snapshot of a [`MemStorage`], taken at
/// [`ReadStorage::snapshot`] time. `max_id` and `root` are captured
/// when the snapshot is created, so later appends to the writer are invisible
/// through it; the base pool itself is shared read-only.
#[derive(Debug, Clone)]
pub struct MemReadStorage {
    bases: Arc<RwLock<Vec<SlotBase>>>,
    status: StorageHead,
}

impl VecBases for MemReadStorage {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase>>> {
        &self.bases
    }
}

impl StoreRead for MemReadStorage {
    fn status(&self) -> StorageHead {
        self.status
    }
}

impl ReadStorage for MemReadStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}
