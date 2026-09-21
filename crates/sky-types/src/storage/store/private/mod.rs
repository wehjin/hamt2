use crate::trie::{HandleTrieConfig, SlotBase};
use std::sync::{Arc, RwLock};

pub trait InternalStoreRead {
    /// Get reference to bases.
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<HandleTrieConfig>>>>;
}
