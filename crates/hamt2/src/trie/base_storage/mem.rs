use crate::trie::base::{Base, BaseId};
use crate::trie::base_storage::errors::{BaseStorageReadError, BaseStorageWriteError};
use crate::trie::base_storage::{
    BaseStorageRead, BaseStorageReadWrite,
};
use crate::trie::core::map_base::MapBase;
use serde::{Deserialize, Serialize};

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that `BaseId(0)` always
/// reads back the empty base and no storage is wasted storing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemBaseStorage {
    bases: Vec<Base>,
    root: Option<MapBase>,
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
            root: None,
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

    async fn read_root(&self) -> Result<Option<MapBase>, BaseStorageReadError> {
        Ok(self.root.clone())
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

    async fn write_root(&mut self, root: MapBase) -> Result<(), BaseStorageWriteError> {
        self.root = Some(root);
        Ok(())
    }
}
