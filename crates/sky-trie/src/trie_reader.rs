use sky_types::storage::{ReadStorage, ReadStorageError, StorageStatus};
use sky_types::trie::MapBase;
use sky_types::trie::TrieStream;
use sky_types::trie::{SlotBase, SlotBaseId, TrieRead};

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug, Clone)]
pub struct TrieReader<S: ReadStorage + TrieRead + Clone + Send> {
    storage: S,
}

impl<S: ReadStorage + TrieRead + Clone + Send> TrieStream for TrieReader<S> {
    type Subtrie = TrieReader<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl<S: ReadStorage + TrieRead + Clone + Send> ReadStorage for TrieReader<S> {
    type Snapshot = TrieReader<S>;

    fn status(&self) -> StorageStatus {
        self.storage.status()
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let storage = self.storage.with_new_root(new_root);
        Self { storage }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl<S: ReadStorage + TrieRead + Clone + Send> TrieRead for TrieReader<S> {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.storage.read_base(id).await
    }
    fn read_root(&self) -> MapBase {
        self.storage.read_root()
    }
}

impl<S: ReadStorage + TrieRead + Clone + Send> TrieReader<S> {
    /// Builds a reader over the given storage with the given root.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        Self { storage }
    }
}
