use crate::storage_trie_query::StorageTrieQuery;
use crate::storage::ReadStorage;
use sky_types::storage::error::StorageReadError;
use sky_types::trie::MapBase;
use sky_types::trie::TrieValue;

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug)]
pub struct TrieReader<S: ReadStorage> {
    pub(crate) root: MapBase,
    storage: S,
}

impl<S: ReadStorage> TrieReader<S> {
    /// Builds a reader over the given storage with the given root.
    pub fn new(root: MapBase, storage: S) -> Self {
        Self { root, storage }
    }

    /// Connects to the storage, loading the persisted root.
    pub async fn connect(storage: S) -> Result<Self, StorageReadError> {
        let root = storage.read_root().await?;
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

impl<S: ReadStorage> StorageTrieQuery<S> for TrieReader<S> {
    fn storage(&self) -> &S {
        &self.storage
    }
}
