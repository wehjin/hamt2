use crate::trie::TrieQuery;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::base_storage::errors::{BaseStorageReadError, BaseStorageWriteError};
use crate::trie::core::map_base::MapBase;
use trie_ref::TrieRef;

pub mod deep;
pub mod insert;
pub mod query;
pub mod readonly;
pub mod trie_ref;

#[derive(Debug)]
pub struct Trie<S: BaseStorageReadWrite> {
    root: MapBase,
    storage: S,
}

impl<S: BaseStorageReadWrite> TrieQuery<S> for Trie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        &self.storage
    }
}

impl<S: BaseStorageReadWrite> Trie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, BaseStorageReadError> {
        let root = storage.get_root().await?;
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

    /// A borrowed read-only view of this trie.
    pub fn view(&self) -> TrieRef<'_, S> {
        TrieRef::new(self.root.clone(), &self.storage)
    }
}
