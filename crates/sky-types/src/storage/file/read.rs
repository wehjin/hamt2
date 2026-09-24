use crate::storage::file::edit::FileStorage;
use crate::storage::file::internal;
use crate::storage::{ReadStorageError, StorageStatus, TrieView};
use crate::trie::{Base, BaseId, BaseRead, MapBase};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileView {
    pub(crate) bases_dir: PathBuf,
    pub(crate) status: StorageStatus,
}

impl From<FileStorage> for FileView {
    fn from(storage: FileStorage) -> Self {
        storage.inner.clone()
    }
}

impl BaseRead for FileView {
    fn read_root(&self) -> MapBase {
        self.status.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        assert!(
            id <= self.max_id(),
            "base id {id} is beyond this snapshot's max_id"
        );
        self.read_base_unchecked(id)
    }
}

impl TrieView for FileView {
    type Snapshot = FileView;

    fn status(&self) -> StorageStatus {
        self.status
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let snap_status = self.status.with_new_root(new_root);
        Self {
            status: snap_status,
            ..self
        }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl FileView {
    /// Reads and decodes the base file for `id`, without checking `max_id`.
    /// Base id [`BaseId::ZERO`] reads back the empty base.
    fn read_base_unchecked(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id == BaseId::EMPTY {
            return Ok(Base::empty());
        }
        let path = internal::base_path(&self.bases_dir, id);
        let bytes = std::fs::read(&path).map_err(|e| ReadStorageError::Io(id, e))?;
        postcard::from_bytes::<Base>(&bytes).map_err(|e| ReadStorageError::Decode(id, e))
    }
}
