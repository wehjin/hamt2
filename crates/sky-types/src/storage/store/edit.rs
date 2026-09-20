use crate::storage::store::private::InternalStoreRead;
use crate::storage::{Store, StoreConfig, StoreEditError, StoreRead};
use crate::trie::{SlotBase, SlotBaseId};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct StoreMut<C: StoreConfig> {
    pub(crate) bases: Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>>,
    pub(crate) start_max: SlotBaseId,
    pub(crate) max_id: SlotBaseId,
}

impl<C: StoreConfig> InternalStoreRead<C> for StoreMut<C> {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<C::TrieConfig>>>> {
        &self.bases
    }
}

impl<C: StoreConfig> StoreRead<C> for StoreMut<C> {
    fn max_id(&self) -> SlotBaseId {
        self.max_id
    }
}

impl<C: StoreConfig> StoreMut<C> {
    pub fn add_base(
        &mut self,
        base: SlotBase<C::TrieConfig>,
    ) -> Result<SlotBaseId, StoreEditError> {
        if self.max_id.0 as usize == C::MAX {
            Err(StoreEditError::NoSlotsAvailable)
        } else {
            let next_id = self.max_id + 1;
            debug_assert_eq!(next_id.0 as usize, self.bases.read().unwrap().len());
            let mut write = self.bases.write().unwrap();
            write.push(base);
            self.max_id = next_id;
            Ok(next_id)
        }
    }
    pub fn commit(self) -> Store<C> {
        Store {
            bases: self.bases,
            max_id: self.max_id,
        }
    }
    pub fn rewind(self) -> Store<C> {
        let bases = self.bases;
        {
            let mut write = bases.write().unwrap();
            write.truncate(self.start_max.0 as usize + 1);
        }
        Store {
            bases,
            max_id: self.start_max,
        }
    }
}
