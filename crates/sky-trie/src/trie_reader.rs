use crate::storage_trie_query::StorageTrieQuery;
use sky_types::storage::ReadStorage;
use sky_types::trie::TrieValue;
use sky_types::trie::{MapBase, SlotBaseId};

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug)]
pub struct TrieReader<S: ReadStorage> {
    pub(crate) root: MapBase<SlotBaseId>,
    storage: S,
}

impl<S: ReadStorage> TrieReader<S> {
    /// Builds a reader over the given storage with the given root.
    pub fn new(root: MapBase<SlotBaseId>, storage: S) -> Self {
        Self { root, storage }
    }

    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        let root = storage.read_root();
        Self { root, storage }
    }

    pub fn subtrie_from_value(value: TrieValue<SlotBaseId>, storage: S) -> Option<Self> {
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
