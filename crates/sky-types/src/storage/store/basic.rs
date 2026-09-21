use crate::storage::StorageHead;
use crate::storage::store::edit::StoreMut;
use crate::storage::store::private::InternalStoreRead;
use crate::storage::store::traits::{StoreConfig, StoreRead};
use crate::trie::SlotBase;
use std::sync::{Arc, RwLock};

/// A basic reader for the store. It deliberately does not implement
/// clone because it can switch into a StoreMut and back.
#[derive(Debug)]
pub struct Store<C: StoreConfig> {
    pub(crate) bases: Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>>,
    pub(crate) status: StorageHead<C::TrieConfig>,
}

/// The default base list has an empty `SlotBase` at position
/// `SlotBaseId(0)`.
impl<C: StoreConfig> Default for Store<C> {
    fn default() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![SlotBase::new()])),
            status: StorageHead::default(),
        }
    }
}
impl<C: StoreConfig> InternalStoreRead<C> for Store<C> {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>> {
        &self.bases
    }
}
impl<C: StoreConfig> StoreRead<C> for Store<C> {
    fn status(&self) -> &StorageHead<C::TrieConfig> {
        &self.status
    }
}

impl<C: StoreConfig> Store<C> {
    pub fn into_mut(self) -> StoreMut<C> {
        let Store { bases, status } = self;
        let start_status = status.clone();
        StoreMut {
            bases,
            start_status,
            status,
        }
    }
}
