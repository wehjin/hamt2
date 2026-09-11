use crate::trie::base::{Base, BaseId};
use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadError, BaseStorageReadWrite, BaseStorageWriteError};
use std::future;
use std::sync::{Arc, RwLock};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that `BaseId(0)` always
/// reads back the empty base and no storage is wasted storing it.
#[derive(Debug, Clone)]
pub struct MemBaseStorage {
    bases: Vec<Base>,
}

impl MemBaseStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemBaseStorage {
    fn default() -> Self {
        Self {
            bases: vec![Base::new()],
        }
    }
}

impl BaseStorageRead for MemBaseStorage {
    async fn read(&self, id: BaseId) -> Result<Base, BaseStorageReadError> {
        let index = id.0 as usize;
        let base = self.bases[index].clone();
        Ok(base)
    }

    fn max_id(&self) -> Option<BaseId> {
        let len = self.bases.len();
        if len <= 1 {
            None
        } else {
            Some(BaseId((len - 1) as i32))
        }
    }
}

impl BaseStorageReadWrite for MemBaseStorage {
    fn next_id(&self) -> BaseId {
        BaseId(self.bases.len() as i32)
    }

    async fn append(&mut self, base: &Base) -> Result<BaseId, BaseStorageWriteError> {
        let id = self.next_id();
        self.bases.push(base.clone());
        Ok(id)
    }
}

/// A `MemBaseStorage` wrapped in an interior-mutable shared arena so that
/// trie clones and sub-tries all read and append to the same bases.
#[derive(Debug, Clone, Default)]
pub struct SharedMemBaseStorage(Arc<RwLock<MemBaseStorage>>);

impl SharedMemBaseStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_storage(storage: MemBaseStorage) -> Self {
        Self(Arc::new(RwLock::new(storage)))
    }
}

impl BaseStorageRead for SharedMemBaseStorage {
    async fn read(&self, id: BaseId) -> Result<Base, BaseStorageReadError> {
        let storage = self.0.read().expect("storage poisoned");
        let base = storage.bases[id.0 as usize].clone();
        Ok(base)
    }

    fn max_id(&self) -> Option<BaseId> {
        self.0.read().expect("storage poisoned").max_id()
    }
}

impl BaseStorageReadWrite for SharedMemBaseStorage {
    fn next_id(&self) -> BaseId {
        self.0.read().expect("storage poisoned").next_id()
    }

    fn append(
        &mut self,
        base: &Base,
    ) -> impl Future<Output = Result<BaseId, BaseStorageWriteError>> {
        let mut storage = self.0.write().expect("storage poisoned");
        let id = storage.next_id();
        storage.bases.push(base.clone());
        future::ready(Ok(id))
    }
}