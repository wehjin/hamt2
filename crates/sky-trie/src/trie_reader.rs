use crate::storage_trie_query::StorageTrieQuery;
use crate::trie_storage::ReadTrieStorage;
use crate::trie_storage::errors::StorageReadError;
use sky_types::trie::MapBase;
use sky_types::trie::TrieValue;

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug)]
pub struct TrieReader<S: ReadTrieStorage> {
    pub(crate) root: MapBase,
    storage: S,
}

impl<S: ReadTrieStorage> TrieReader<S> {
    /// Builds a reader over the given storage with the given root.
    pub fn new(root: MapBase, storage: S) -> Self {
        Self { root, storage }
    }

    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, StorageReadError> {
        let root = storage.get_root().await?;
        Ok(Self { root, storage })
    }

    pub fn subtrie_from_value(value: TrieValue, storage: S) -> Option<Self> {
        let root = match value {
            TrieValue::SubTrie(root) => root,
            TrieValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }
}

impl<S: ReadTrieStorage> StorageTrieQuery<S> for TrieReader<S> {
    fn storage(&self) -> &S {
        &self.storage
    }
}
