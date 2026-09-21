use sky_types::storage::{ReadStorage, Storage, StorageHead, StoreRead};
use sky_types::trie::MapBase;
use sky_types::trie::TrieValue;
use sky_types::trie::{SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError};

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug)]
pub struct TrieReader<S: ReadStorage + TrieBaseRead> {
    pub(crate) root: MapBase,
    storage: S,
}

impl<S: ReadStorage + TrieBaseRead> StoreRead for TrieReader<S> {
    fn status(&self) -> StorageHead {
        self.storage.status()
    }
    fn read_root(&self) -> MapBase {
        self.root
    }
}

impl<S: ReadStorage + TrieBaseRead> TrieBaseRead for TrieReader<S> {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
        self.storage.read_base(id).await
    }
}

impl<S: ReadStorage + TrieBaseRead> Storage<S> for TrieReader<S> {
    fn storage(&self) -> &S {
        &self.storage
    }
}

impl<S: ReadStorage + TrieBaseRead> TrieReader<S> {
    /// Builds a reader over the given storage with the given root.
    pub fn new(root: MapBase, storage: S) -> Self {
        Self { root, storage }
    }

    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        let root = storage.read_root();
        Self { root, storage }
    }

    pub fn subtrie_from_value(value: TrieValue, storage: S) -> Option<Self> {
        let root = match value {
            TrieValue::SubTrie(root) => root,
            TrieValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }
}
