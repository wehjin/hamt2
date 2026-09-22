use crate::storage::StorageHead;
use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieQueryError, TrieRead};
use std::sync::{Arc, RwLock};

pub trait StoreConfig {
    const MAX: usize;
}

pub trait VecBases {
    /// Get reference to bases.
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase>>>;
}

pub trait StoreRead {
    fn status(&self) -> StorageHead;

    /// Get the maximum reading id.
    fn max_id(&self) -> SlotBaseId {
        self.status().max_id
    }
}

impl<T: StoreRead + VecBases> TrieRead for T {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
        assert!(id <= self.max_id(), "id out of bounds {id:?}");
        let index = id.0 as usize;
        let read = self.bases().read().unwrap();
        let base = read[index].clone();
        Ok(base)
    }

    fn read_root(&self) -> MapBase {
        self.status().root
    }
}
