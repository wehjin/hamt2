use crate::TrieReader;
use sky_types::storage::error::WriteStorageError;
use sky_types::storage::{ReadStorageError, StorageStatus, BaseEdit, BaseView};
use sky_types::trie::{Base, BaseId, BaseRead, MapBase};
use sky_types::trie::{BaseCommit, TrieStream};

#[derive(Debug)]
pub struct Trie<S: BaseEdit> {
    pub(crate) storage: S,
}

impl<S: BaseEdit> TrieStream for Trie<S> {
    type Subtrie = TrieReader<S::Snapshot>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.snapshot().to_subtrie(subtrie_root)
    }
}
impl<S: BaseEdit> BaseView for Trie<S> {
    type Snapshot = TrieReader<S::Snapshot>;

    fn status(&self) -> StorageStatus {
        self.storage.status()
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let storage = self.storage.with_new_root(new_root);
        Self { storage }
    }

    fn snapshot(&self) -> Self::Snapshot {
        let snap_storage = self.storage.snapshot();
        TrieReader::new(snap_storage)
    }
}
impl<S: BaseEdit> BaseRead for Trie<S> {
    fn read_root(&self) -> MapBase {
        self.storage.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.storage.read_base(id).await
    }
}

/// Trie construction methods.
impl<S: BaseEdit> Trie<S> {
    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        Self { storage }
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(self) -> Result<Self, WriteStorageError> {
        // We're writing directly as we go along so there is nothing
        // to do here for now. We can do better by not writing directly
        // to support rewind and compaction.
        Ok(self)
    }

    pub fn close(self) -> S {
        self.storage
    }
}

impl<S: BaseEdit> BaseCommit for Trie<S> {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.storage.commit_root(root).await
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        self.storage.commit_base(base).await
    }
}

impl<S: BaseEdit> BaseEdit for Trie<S> {}
