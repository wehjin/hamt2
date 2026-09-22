use crate::storage::store::edit::StoreMut;
use crate::storage::store::traits::StoreConfig;
use crate::storage::traits::VecBases;
use crate::storage::{StorageHead, StoreRead};
use crate::trie::SlotBase;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

/// A basic reader for the store. It deliberately does not implement
/// clone because it can switch into a StoreMut and back.
#[derive(Debug)]
pub struct Store<C: StoreConfig> {
    pub(crate) bases: Arc<RwLock<Vec<SlotBase>>>,
    pub(crate) status: StorageHead,
    pub(crate) _phantom_data: PhantomData<C>,
}

/// The default base list has an empty `SlotBase` at position
/// `SlotBaseId(0)`.
impl<C: StoreConfig> Default for Store<C> {
    fn default() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![SlotBase::empty()])),
            status: StorageHead::default(),
            _phantom_data: PhantomData,
        }
    }
}
impl<C: StoreConfig> VecBases for Store<C> {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase>>> {
        &self.bases
    }
}
impl<C: StoreConfig> StoreRead for Store<C> {
    fn status(&self) -> StorageHead {
        self.status
    }
}

impl<C: StoreConfig> Store<C> {
    pub fn into_mut(self) -> StoreMut<C> {
        let Store {
            bases,
            status,
            _phantom_data,
        } = self;
        let start_status = status.clone();
        StoreMut {
            bases,
            start_status,
            status,
            _phantom_data,
        }
    }
}
