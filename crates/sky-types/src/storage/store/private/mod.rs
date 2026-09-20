use crate::storage::StoreConfig;
use crate::trie::SlotBase;
use std::sync::{Arc, RwLock};

pub trait InternalStoreRead<C: StoreConfig> {
    /// Get reference to bases.
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>>;
}
