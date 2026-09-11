use crate::trie::base_storage::errors::{BaseStorageReadError, BaseStorageWriteError};
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::map_base::MapBase;

pub mod deep;
pub mod insert;
pub mod query;

#[derive(Debug)]
pub struct Trie<S: BaseStorageReadWrite> {
    root: MapBase,
    storage: S,
}

impl<S: BaseStorageReadWrite> Trie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, BaseStorageReadError> {
        let root = storage
            .read_root()
            .await?
            .unwrap_or_else(|| MapBase::empty());
        Ok(Self { root, storage })
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(mut self) -> Result<Self, BaseStorageWriteError> {
        self.storage.write_root(self.root.clone()).await?;
        Ok(self)
    }

    pub fn unwrap(self) -> MapBase {
        self.root
    }

    pub fn close(self) -> S {
        self.storage
    }
}
