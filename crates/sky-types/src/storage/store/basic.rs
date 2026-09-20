use crate::storage::store::edit::StoreMut;
use crate::storage::store::private::InternalStoreRead;
use crate::storage::store::traits::{StoreConfig, StoreRead};
use crate::trie::{SlotBase, SlotBaseId};
use std::sync::{Arc, RwLock};

/// A basic reader for the store. It deliberately does not implement
/// clone because it can switch into a StoreMut and back.
#[derive(Debug)]
pub struct Store<C: StoreConfig> {
    pub(crate) bases: Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>>,
    pub(crate) max_id: SlotBaseId,
}

/// The default base list has an empty `SlotBase` at position
/// `SlotBaseId(0)`.
impl<C: StoreConfig> Default for Store<C> {
    fn default() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![SlotBase::new()])),
            max_id: SlotBaseId(0),
        }
    }
}
impl<C: StoreConfig> InternalStoreRead<C> for Store<C> {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>> {
        &self.bases
    }
}
impl<C: StoreConfig> StoreRead<C> for Store<C> {
    fn max_id(&self) -> SlotBaseId {
        self.max_id
    }
}

impl<C: StoreConfig> Store<C> {
    pub fn into_mut(self) -> StoreMut<C> {
        let arc = self.bases.clone();
        let start_max = self.max_id;
        StoreMut {
            bases: arc,
            start_max,
            max_id: start_max,
        }
    }
}
