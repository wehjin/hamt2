mod read;
mod write;

use std::sync::{Arc, RwLock};
pub use read::*;
pub use write::*;
use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieQueryError, TrieRead};

pub trait Storage<S> {
    fn storage(&self) -> &S;
}

pub trait VecBases {
    /// Get reference to bases.
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase>>>;
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