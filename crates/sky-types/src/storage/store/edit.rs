use crate::storage::store::private::InternalStoreRead;
use crate::storage::{StorageHead, Store, StoreConfig, StoreEditError, StoreRead};
use crate::trie::{HandleTrieConfig, MapBase, SlotBase, SlotBaseId};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct StoreMut<C: StoreConfig> {
    pub(crate) bases: Arc<RwLock<Vec<SlotBase<HandleTrieConfig>>>>,
    pub(crate) start_status: StorageHead<HandleTrieConfig>,
    pub(crate) status: StorageHead<HandleTrieConfig>,
    pub(crate) _phantom_data: PhantomData<C>,
}

impl<C: StoreConfig> InternalStoreRead for StoreMut<C> {
    fn bases(&self) -> &Arc<RwLock<Vec<SlotBase<HandleTrieConfig>>>> {
        &self.bases
    }
}

impl<C: StoreConfig> StoreRead for StoreMut<C> {
    fn status(&self) -> &StorageHead<HandleTrieConfig> {
        &self.status
    }
}

impl<C: StoreConfig> StoreMut<C> {
    pub fn add_base(
        &mut self,
        base: SlotBase<HandleTrieConfig>,
    ) -> Result<SlotBaseId, StoreEditError> {
        if self.status.max_id.0 as usize == C::MAX {
            Err(StoreEditError::NoSlotsAvailable)
        } else {
            let next_id = self.status.max_id + 1;
            debug_assert_eq!(next_id.0 as usize, self.bases.read().unwrap().len());
            let mut write = self.bases.write().unwrap();
            write.push(base);
            self.status.max_id = next_id;
            Ok(next_id)
        }
    }
    pub fn write_root(&mut self, root: MapBase<HandleTrieConfig>) -> Result<(), StoreEditError> {
        self.status.root = root;
        Ok(())
    }
    pub fn commit(self) -> Store<C> {
        Store {
            bases: self.bases,
            status: self.status,
            _phantom_data: self._phantom_data,
        }
    }
    pub fn rewind(self) -> Store<C> {
        let start_bases = {
            let bases = self.bases;
            {
                let mut write = bases.write().unwrap();
                write.truncate(self.start_status.max_id.0 as usize + 1);
            }
            bases
        };
        Store {
            bases: start_bases,
            status: self.start_status,
            _phantom_data: self._phantom_data,
        }
    }
}
