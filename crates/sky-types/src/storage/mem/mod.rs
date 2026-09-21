use crate::storage::{ReadStorage, ReadStorageError, ReadWriteStorage, WriteStorageError};
use crate::trie::{MapBase, SlotBase, SlotBaseId};
use std::sync::{Arc, RwLock};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that
/// [`SlotBaseId::ZERO`] always reads back the empty base and no storage is
/// wasted storing it.
#[derive(Debug, Clone)]
pub struct MemStorage {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    bases: Vec<SlotBase>,
    root: MapBase,
}

impl Inner {
    fn new() -> Self {
        Self {
            bases: vec![SlotBase::new()],
            root: MapBase::empty(),
        }
    }
}

impl MemStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::new())),
        }
    }
}

impl ReadStorage for MemStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        let inner = self.inner.read().expect("storage poisoned");
        MemReadStorage {
            inner: Arc::clone(&self.inner),
            max_id: (inner.bases.len() - 1) as i32,
            root: inner.root.clone(),
        }
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        let inner = self.inner.read().expect("storage poisoned");
        assert!(
            id.0 < inner.bases.len() as i32,
            "base id {id} has not been written"
        );
        let index = id.0 as usize;
        Ok(inner.bases[index].clone())
    }

    fn max_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId((inner.bases.len() - 1) as i32)
    }

    fn read_root(&self) -> MapBase {
        self.inner.read().expect("storage poisoned").root
    }
}

/// A read-only snapshot of a [`MemStorage`], taken at
/// [`ReadStorage::snapshot`] time. `max_id` and `root` are captured
/// when the snapshot is created, so later appends to the writer are invisible
/// through it; the base pool itself is shared read-only.
#[derive(Debug, Clone)]
pub struct MemReadStorage {
    inner: Arc<RwLock<Inner>>,
    max_id: i32,
    root: MapBase,
}

impl ReadStorage for MemReadStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        assert!(
            id.0 <= self.max_id,
            "base id {id} is beyond this snapshot's max_id"
        );
        let inner = self.inner.read().expect("storage poisoned");
        let index = id.0 as usize;
        Ok(inner.bases[index].clone())
    }

    fn max_id(&self) -> SlotBaseId {
        SlotBaseId(self.max_id)
    }

    fn read_root(&self) -> MapBase {
        self.root
    }
}

impl ReadWriteStorage for MemStorage {
    fn next_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId(inner.bases.len() as i32)
    }

    async fn append(
        &mut self,
        base: &SlotBase,
    ) -> Result<SlotBaseId, WriteStorageError> {
        let mut inner = self.inner.write().expect("storage poisoned");
        let id = inner.bases.len() as i32;
        inner.bases.push(base.clone());
        Ok(SlotBaseId(id))
    }

    async fn write_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.inner.write().expect("storage poisoned").root = root;
        Ok(())
    }
}
