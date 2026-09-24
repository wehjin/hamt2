use crate::storage::{
    FileReadStorage, ReadStorageError, StorageStatus, TrieEdit, TrieView, WriteStorageError, file,
};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// A file-backed storage for Bases.
///
/// Bases are stored in a `bases` subfolder of the given folder, spread over a
/// two-level subfolder structure so no directory grows unbounded:
///
/// ```text
/// <folder>/
///   max_id          (file holding the highest written base id)
///   root            (file holding the committed root map base)
///   bases/
///     <level-1>/
///       <level-2>/
///         <id>.postcard
/// ```
///
/// Base id [`BaseId::ZERO`] is the reserved empty base and is never written to disk.
#[derive(Debug)]
pub struct FileStorage {
    pub(crate) inner: FileReadStorage,
    max_id_path: PathBuf,
    root_path: PathBuf,
}

impl BaseRead for FileStorage {
    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }
}

impl TrieView for FileStorage {
    type Snapshot = FileReadStorage;

    fn status(&self) -> StorageStatus {
        self.inner.status()
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let inner = self.inner.with_new_root(new_root);
        Self { inner, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }
}

impl FileStorage {
    pub(crate) const BASES_DIR: &'static str = "bases";
    pub(crate) const MAX_ID_FILE: &'static str = "max_id";
    pub(crate) const ROOT_FILE: &'static str = "root";

    /// Creates a fresh empty storage in the given folder.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = path.as_ref();
        let bases_dir = root.join(Self::BASES_DIR);
        let max_id_path = root.join(Self::MAX_ID_FILE);
        let root_path = root.join(Self::ROOT_FILE);
        std::fs::create_dir_all(&bases_dir)?;
        let status = StorageStatus::default();

        file::write_max_id_file(&max_id_path, status.max_id).map_err(file::write_to_io)?;
        file::write_root_file(&root_path, &status.root).map_err(file::write_to_io)?;
        let inner = FileReadStorage { bases_dir, status };
        Ok(Self {
            inner,
            max_id_path,
            root_path,
        })
    }

    /// Opens an existing storage. A missing folder or max id file is treated
    /// as an empty storage.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = path.as_ref();
        let bases_dir = root.join(Self::BASES_DIR);
        let max_id_path = root.join(Self::MAX_ID_FILE);
        let root_path = root.join(Self::ROOT_FILE);
        std::fs::create_dir_all(&bases_dir)?;
        let max_id = match std::fs::read(&max_id_path) {
            Ok(bytes) => postcard::from_bytes::<i32>(&bytes)
                .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?,
            Err(e) if e.kind() == ErrorKind::NotFound => 0,
            Err(e) => return Err(e),
        };
        let root = file::read_root_file(&root_path).map_err(file::read_to_io)?;
        let status = StorageStatus {
            max_id: BaseId(max_id),
            root,
        };
        let inner = FileReadStorage { bases_dir, status };
        Ok(Self {
            inner,
            max_id_path,
            root_path,
        })
    }
}

impl TrieEdit for FileStorage {
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }
}

impl BaseCommit for FileStorage {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        match file::write_root_file(&self.root_path, &root) {
            Ok(()) => {
                self.inner.status.root = root;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let id = self.next_id();
        let bytes = match postcard::to_allocvec(&base) {
            Ok(bytes) => bytes,
            Err(e) => return Err(WriteStorageError::Encode(id, e)),
        };
        let path = file::base_path(&self.inner.bases_dir, id);
        if let Err(e) = std::fs::create_dir_all(path.parent().expect("base path has parent")) {
            return Err(WriteStorageError::Io(id, e));
        }
        if let Err(e) = std::fs::write(&path, bytes) {
            return Err(WriteStorageError::Io(id, e));
        }
        if let Err(e) = file::write_max_id_file(&self.max_id_path, id) {
            return Err(e);
        }
        self.inner.status.max_id = id;
        Ok(id)
    }
}
