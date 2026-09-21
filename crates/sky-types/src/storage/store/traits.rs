use crate::storage::StorageHead;
use crate::trie::{MapBase, RootTrieQuery, SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError};
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

    /// Get the current root.
    fn read_root(&self) -> MapBase {
        self.status().root
    }
}

impl<T: StoreRead> RootTrieQuery for T {
    fn root(&self) -> MapBase {
        self.read_root()
    }
}

impl<T: StoreRead + VecBases> TrieBaseRead for T {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
        assert!(id <= self.max_id(), "id out of bounds {id:?}");
        let index = id.0 as usize;
        let read = self.bases().read().unwrap();
        let base = read[index].clone();
        Ok(base)
    }
}
