use crate::storage::{ReadStorage, ReadStorageError, StorageStatus};
use crate::trie::{Base, BaseId, MapBase, RootBaseRead};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct MemReadStorage {
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) status: StorageStatus,
}

impl MemReadStorage {
    pub fn empty() -> Self {
        let bases = Arc::new(RwLock::new(vec![Base::empty()]));
        let status = StorageStatus::default();
        Self { bases, status }
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

impl RootBaseRead for MemReadStorage {
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
