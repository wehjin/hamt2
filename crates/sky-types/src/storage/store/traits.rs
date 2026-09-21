use crate::storage::StorageHead;
use crate::storage::store::private::InternalStoreRead;
use crate::trie::{
    HandleTrieConfig, MapBase, RootTrieQuery, SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError,
};

pub trait StoreConfig {
    const MAX: usize;
}

pub trait StoreRead: InternalStoreRead {
    fn status(&self) -> &StorageHead<HandleTrieConfig>;

    /// Get the maximum reading id.
    fn max_id(&self) -> SlotBaseId {
        self.status().max_id.clone()
    }

    /// Get the current root.
    fn read_root(&self) -> &MapBase<HandleTrieConfig> {
        &self.status().root
    }
}

impl<T: StoreRead> RootTrieQuery<HandleTrieConfig> for T {
    fn root(&self) -> &MapBase<HandleTrieConfig> {
        self.read_root()
    }
}

impl<T: StoreRead> TrieBaseRead for T {
    type Config = HandleTrieConfig;

    async fn read_base(
        &self,
        id: SlotBaseId,
    ) -> Result<SlotBase<HandleTrieConfig>, TrieQueryError> {
        assert!(id <= self.max_id(), "id out of bounds");
        let index = id.0 as usize;
        let read = self.bases().read().unwrap();
        let base = read[index].clone();
        Ok(base)
    }
}
