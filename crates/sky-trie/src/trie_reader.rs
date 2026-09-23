use sky_types::storage::{ReadStorageError, StorageStatus, TrieView};
use sky_types::trie::MapBase;
use sky_types::trie::TrieStream;
use sky_types::trie::{Base, BaseId, BaseRead};

/// A read-only trie over an owned read-only storage, used only for queries.
#[derive(Debug, Clone)]
pub struct TrieReader<S: TrieView + BaseRead + Clone + Send> {
    storage: S,
}

impl<S: TrieView + BaseRead + Clone + Send> TrieReader<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
}

impl<S: TrieView + BaseRead + Clone + Send> TrieStream for TrieReader<S> {
    type Subtrie = TrieReader<S>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl<S: TrieView + BaseRead + Clone + Send> TrieView for TrieReader<S> {
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

impl<S: TrieView + BaseRead + Clone + Send> BaseRead for TrieReader<S> {
    fn read_root(&self) -> MapBase {
        self.storage.read_root()
    }
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.storage.read_base(id).await
    }
}
