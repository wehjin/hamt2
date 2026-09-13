use crate::trie::TrieQuery;
use crate::trie::trie_storage::ReadWriteTrieStorage;
use crate::trie::trie_storage::errors::{TrieStorageReadError, TrieStorageWriteError};
use crate::trie::types::map_base::MapBase;
use trie_ref::TrieRef;

pub mod deep;
pub mod insert;
pub mod query;
pub mod readonly;
pub mod trie_ref;

#[derive(Debug)]
pub struct Trie<S: ReadWriteTrieStorage> {
    root: MapBase,
    storage: S,
}

impl<S: ReadWriteTrieStorage> TrieQuery<S> for Trie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        &self.storage
    }
}

impl<S: ReadWriteTrieStorage> Trie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, TrieStorageReadError> {
        let root = storage.get_root().await?;
        Ok(Self { root, storage })
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(mut self) -> Result<Self, TrieStorageWriteError> {
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
