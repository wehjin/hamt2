use super::query::TrieQuery;
use crate::trie::trie_storage::ReadTrieStorage;
use crate::trie::trie_storage::errors::TrieStorageReadError;
use crate::trie::types::map_base::MapBase;

/// A read-only trie over a read-only storage, used only for queries.
#[derive(Debug)]
pub struct ReadTrie<S: ReadTrieStorage> {
    root: MapBase,
    storage: S,
}

impl<S: ReadTrieStorage> ReadTrie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, TrieStorageReadError> {
        let root = storage.get_root().await?;
        Ok(Self { root, storage })
    }
}

impl<S: ReadTrieStorage> TrieQuery<S> for ReadTrie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        &self.storage
    }
}
