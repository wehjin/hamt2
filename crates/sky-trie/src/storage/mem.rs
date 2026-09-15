use crate::storage::{ReadTrieStorage, ReadWriteTrieStorage};
use crate::types::slot_base::SlotBase;
use sky_types::storage::error::{StorageReadError, StorageWriteError};
use sky_types::trie::MapBase;
use sky_types::trie::SlotBaseId;
use std::sync::{Arc, RwLock};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that `BaseId(0)` always
/// reads back the empty base and no storage is wasted storing it.
#[derive(Debug, Clone)]
pub struct MemTrieStorage {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    bases: Vec<SlotBase>,
    root: Option<MapBase>,
}

impl Inner {
    fn new() -> Self {
        Self {
            bases: vec![SlotBase::new()],
            root: None,
        }
    }
}

impl MemTrieStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemTrieStorage {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner::new())),
        }
    }
}

impl ReadTrieStorage for MemTrieStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        let inner = self.inner.read().expect("storage poisoned");
        MemReadStorage {
            inner: Arc::clone(&self.inner),
            max_id: (inner.bases.len() - 1) as i32,
            root: inner.root.clone(),
        }
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, StorageReadError> {
        let inner = self.inner.read().expect("storage poisoned");
        let index = id.0 as usize;
        Ok(inner.bases[index].clone())
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        let inner = self.inner.read().expect("storage poisoned");
        let len = inner.bases.len();
        if len <= 1 {
            None
        } else {
            Some(SlotBaseId((len - 1) as i32))
        }
    }

    async fn read_root(&self) -> Result<Option<MapBase>, StorageReadError> {
        Ok(self.inner.read().expect("storage poisoned").root.clone())
    }
}

/// A read-only snapshot of a [`MemTrieStorage`], taken at
/// [`ReadTrieStorage::snapshot`] time. `max_id` and `root` are captured
/// when the snapshot is created, so later appends to the writer are invisible
/// through it; the base pool itself is shared read-only.
#[derive(Debug, Clone)]
pub struct MemReadStorage {
    inner: Arc<RwLock<Inner>>,
    max_id: i32,
    root: Option<MapBase>,
}

impl ReadTrieStorage for MemReadStorage {
    type Snapshot = MemReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, StorageReadError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        if id.0 > self.max_id {
            return Err(StorageReadError::NotFound(id));
        }
        let inner = self.inner.read().expect("storage poisoned");
        let index = id.0 as usize;
        Ok(inner.bases[index].clone())
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        if self.max_id == 0 {
            None
        } else {
            Some(SlotBaseId(self.max_id))
        }
    }

    async fn read_root(&self) -> Result<Option<MapBase>, StorageReadError> {
        Ok(self.root.clone())
    }
}

impl ReadWriteTrieStorage for MemTrieStorage {
    fn next_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId(inner.bases.len() as i32)
    }

    async fn append(&mut self, base: &SlotBase) -> Result<SlotBaseId, StorageWriteError> {
        let mut inner = self.inner.write().expect("storage poisoned");
        let id = inner.bases.len() as i32;
        inner.bases.push(base.clone());
        Ok(SlotBaseId(id))
    }

    async fn write_root(&mut self, root: MapBase) -> Result<(), StorageWriteError> {
        self.inner.write().expect("storage poisoned").root = Some(root);
        Ok(())
    }
}
