use crate::trie_storage::errors::{TrieStorageReadError, TrieStorageWriteError};
use crate::trie_storage::{ReadTrieStorage, ReadWriteTrieStorage};
use crate::types::slot_base::SlotBase;
use serde::{Deserialize, Serialize};
use sky_types::trie::map_base::MapBase;
use sky_types::trie::slot_base_id::SlotBaseId;

/// An in-memory storage for Bases backed by a `Vec<Base>`.
///
/// The vec is seeded with the empty base at index 0 so that `BaseId(0)` always
/// reads back the empty base and no storage is wasted storing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemTrieStorage {
    bases: Vec<SlotBase>,
    root: Option<MapBase>,
}

impl MemTrieStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for MemTrieStorage {
    fn default() -> Self {
        Self {
            bases: vec![SlotBase::new()],
            root: None,
        }
    }
}

impl ReadTrieStorage for MemTrieStorage {
    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, TrieStorageReadError> {
        let index = id.0 as usize;
        let base = self.bases[index].clone();
        Ok(base)
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        let len = self.bases.len();
        if len <= 1 {
            None
        } else {
            Some(SlotBaseId((len - 1) as i32))
        }
    }

    async fn read_root(&self) -> Result<Option<MapBase>, TrieStorageReadError> {
        Ok(self.root.clone())
    }
}

/// A read-only snapshot of a [`MemTrieStorage`], taken at
/// `BaseStorageReadWrite::to_readonly` time. Owns a full copy of the data, so
/// later appends to the writer are invisible through it.
#[derive(Debug, Clone)]
pub struct MemReadStorage(MemTrieStorage);

impl ReadTrieStorage for MemReadStorage {
    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, TrieStorageReadError> {
        self.0.read(id).await
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        self.0.max_id()
    }

    async fn read_root(&self) -> Result<Option<MapBase>, TrieStorageReadError> {
        self.0.read_root().await
    }
}

impl ReadWriteTrieStorage for MemTrieStorage {
    type ReadOnly = MemReadStorage;

    fn next_id(&self) -> SlotBaseId {
        SlotBaseId(self.bases.len() as i32)
    }

    async fn append(&mut self, base: &SlotBase) -> Result<SlotBaseId, TrieStorageWriteError> {
        let id = self.next_id();
        self.bases.push(base.clone());
        Ok(id)
    }

    async fn write_root(&mut self, root: MapBase) -> Result<(), TrieStorageWriteError> {
        self.root = Some(root);
        Ok(())
    }

    fn to_readonly(&self) -> Self::ReadOnly {
        MemReadStorage(self.clone())
    }
}
