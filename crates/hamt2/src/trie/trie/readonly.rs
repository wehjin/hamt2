use crate::trie::base_storage::errors::BaseStorageReadError;
use crate::trie::base_storage::BaseStorageRead;
use crate::trie::core::map_base::MapBase;
use super::query::TrieQuery;

/// A read-only trie over a read-only storage, used only for queries.
#[derive(Debug)]
pub struct ReadTrie<S: BaseStorageRead> {
    root: MapBase,
    storage: S,
}

impl<S: BaseStorageRead> ReadTrie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, BaseStorageReadError> {
        let root = storage.get_root().await?;
        Ok(Self { root, storage })
    }
}

impl<S: BaseStorageRead> TrieQuery<S> for ReadTrie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        &self.storage
    }
}
