use crate::storage::store::private::InternalStoreRead;
use crate::storage::{StorageHead, StoreReadError};
use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieConfig};

pub trait StoreConfig {
    type TrieConfig: TrieConfig;
    const MAX: usize;
}

pub trait StoreRead<C: StoreConfig>: InternalStoreRead<C> {
    fn status(&self) -> &StorageHead<C::TrieConfig>;

    /// Get the maximum reading id.
    fn max_id(&self) -> SlotBaseId {
        self.status().max_id
    }

    /// Get the current root.
    fn read_root(&self) -> MapBase<C::TrieConfig> {
        self.status().root.clone()
    }

    /// Panics when `id` is out of bounds.
    fn read_base(&self, id: SlotBaseId) -> Result<SlotBase<C::TrieConfig>, StoreReadError> {
        assert!(id <= self.max_id(), "id out of bounds");
        let index = id.0 as usize;
        let read = self.bases().read().unwrap();
        Ok(read[index].clone())
    }
}
