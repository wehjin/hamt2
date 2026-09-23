use crate::storage::{
    TrieView, ReadStorageError, TrieEdit, StorageStatus, WriteStorageError,
};
use crate::trie::{Base, BaseId, MapBase, BaseRead, BaseCommit};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// A read-only, immutable view of a [`FileStorage`] taken at
/// [`TrieView::snapshot`] time.
///
/// `max_id` and `root` are captured into memory when the view is created, so
/// later appends or commits on the writer are invisible through it. Bases are
/// still read from disk: they are written once and never modified, so any id
/// at or below the captured `max_id` stays readable.
#[derive(Debug, Clone)]
pub struct FileReadStorage {
    bases_dir: PathBuf,
    status: StorageStatus,
}

impl From<FileStorage> for FileReadStorage {
    fn from(storage: FileStorage) -> Self {
        storage.inner.clone()
    }
}

impl BaseRead for FileReadStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        assert!(
            id <= self.max_id(),
            "base id {id} is beyond this snapshot's max_id"
        );
        self.read_base_unchecked(id)
    }

    fn read_root(&self) -> MapBase {
        self.status.root
    }
}

impl TrieView for FileReadStorage {
    type Snapshot = FileReadStorage;

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

impl FileReadStorage {
    /// Reads and decodes the base file for `id`, without checking `max_id`.
    /// Base id [`BaseId::ZERO`] reads back the empty base.
    fn read_base_unchecked(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id == BaseId::EMPTY {
            return Ok(Base::empty());
        }
        let path = base_path(&self.bases_dir, id);
        let bytes = std::fs::read(&path).map_err(|e| ReadStorageError::Io(id, e))?;
        postcard::from_bytes::<Base>(&bytes).map_err(|e| ReadStorageError::Decode(id, e))
    }
}

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
    inner: FileReadStorage,
    max_id_path: PathBuf,
    root_path: PathBuf,
}

impl BaseRead for FileStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.inner.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
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
    const BASES_DIR: &'static str = "bases";
    const MAX_ID_FILE: &'static str = "max_id";
    const ROOT_FILE: &'static str = "root";

    /// Creates a fresh empty storage in the given folder.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = path.as_ref();
        let bases_dir = root.join(Self::BASES_DIR);
        let max_id_path = root.join(Self::MAX_ID_FILE);
        let root_path = root.join(Self::ROOT_FILE);
        std::fs::create_dir_all(&bases_dir)?;
        let status = StorageStatus::default();

        write_max_id_file(&max_id_path, status.max_id).map_err(write_to_io)?;
        write_root_file(&root_path, &status.root).map_err(write_to_io)?;
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
        let root = read_root_file(&root_path).map_err(read_to_io)?;
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

/// The two-level subfolder layout of a base file under a `bases` dir.
fn base_path(bases_dir: &Path, id: BaseId) -> PathBuf {
    let level_1 = id.0 >> 8;
    let level_2 = id.0 & 0xff;
    bases_dir
        .join(format!("{:04x}", level_1))
        .join(format!("{:04x}", level_2))
        .join(format!("{:08x}.postcard", id.0))
}

/// Writes the highest written base id to the `max_id` file.
fn write_max_id_file(path: &Path, id: BaseId) -> Result<(), WriteStorageError> {
    let bytes = postcard::to_allocvec(&id.0).map_err(|e| WriteStorageError::Encode(id, e))?;
    std::fs::write(path, bytes).map_err(|e| WriteStorageError::Io(id, e))
}

/// Reads the committed root from the `root` file. A missing root file is
/// treated as an empty root.
fn read_root_file(path: &Path) -> Result<MapBase, ReadStorageError> {
    match std::fs::read(path) {
        Ok(bytes) => postcard::from_bytes::<MapBase>(&bytes)
            .map_err(|e| ReadStorageError::Decode(BaseId::ZERO, e)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(MapBase::empty()),
        Err(e) => Err(ReadStorageError::Io(BaseId::ZERO, e)),
    }
}

/// Writes the committed root to the `root` file.
fn write_root_file(path: &Path, root: &MapBase) -> Result<(), WriteStorageError> {
    let bytes =
        postcard::to_allocvec(root).map_err(|e| WriteStorageError::Encode(BaseId::ZERO, e))?;
    std::fs::write(path, bytes).map_err(|e| WriteStorageError::Io(BaseId::ZERO, e))
}

fn write_to_io(e: WriteStorageError) -> std::io::Error {
    match e {
        WriteStorageError::Io(_, e) => e,
        WriteStorageError::Encode(_, e) => std::io::Error::new(ErrorKind::InvalidData, e),
    }
}

fn read_to_io(e: ReadStorageError) -> std::io::Error {
    match e {
        ReadStorageError::Io(_, e) => e,
        ReadStorageError::Decode(_, e) => std::io::Error::new(ErrorKind::InvalidData, e),
    }
}

impl TrieEdit for FileStorage {
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }
}

impl BaseCommit for FileStorage {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        match write_root_file(&self.root_path, &root) {
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
        let path = base_path(&self.inner.bases_dir, id);
        if let Err(e) = std::fs::create_dir_all(path.parent().expect("base path has parent")) {
            return Err(WriteStorageError::Io(id, e));
        }
        if let Err(e) = std::fs::write(&path, bytes) {
            return Err(WriteStorageError::Io(id, e));
        }
        if let Err(e) = write_max_id_file(&self.max_id_path, id) {
            return Err(e);
        }
        self.inner.status.max_id = id;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::map_base::{one_kv, two_kv};
    use crate::trie::{HashKey, BaseRead, TrieValue};

    #[tokio::test]
    async fn empty_storage_max_id_is_zero() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let storage = FileStorage::new(dir.path())?;
        assert_eq!(BaseId::ZERO, storage.max_id());
        assert_eq!(BaseId(1), storage.next_id());
        assert_eq!(
            Base::empty(),
            storage.read_base(BaseId::ZERO).await.expect("read")
        );
        Ok(())
    }

    #[tokio::test]
    async fn append_and_reload_works() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let bases = (1..=10)
            .map(|i| Base::new_kv(HashKey::new(i), TrieValue::U32(i as u32)))
            .collect::<Vec<_>>();
        {
            let mut storage = FileStorage::new(dir.path())?;
            for base in &bases {
                storage.commit_base(base.clone()).await.expect("append");
            }
            assert_eq!(BaseId(10), storage.max_id());
            assert_eq!(BaseId(11), storage.next_id());
            for (i, base) in bases.iter().enumerate() {
                let id = BaseId(i as i32 + 1);
                assert_eq!(base, &storage.read_base(id).await.expect("read"));
            }
        }
        let storage = FileStorage::load(dir.path())?;
        assert_eq!(BaseId(10), storage.max_id());
        for (i, base) in bases.iter().enumerate() {
            let id = BaseId(i as i32 + 1);
            assert_eq!(base, &storage.read_base(id).await.expect("read"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn root_round_trip_works() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let root;
        {
            let mut storage = FileStorage::new(dir.path())?;
            assert_eq!(MapBase::empty(), storage.read_root());
            root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
                .await
                .expect("one_kv");
            storage.commit_root(root.clone()).await.expect("write root");
        }
        let storage = FileStorage::load(dir.path())?;
        assert_eq!(root, storage.read_root());
        Ok(())
    }

    #[tokio::test]
    async fn appending_continues_after_reload() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
        {
            let mut storage = FileStorage::new(dir.path())?;
            storage.commit_base(base.clone()).await.expect("append");
        }
        let mut storage = FileStorage::load(dir.path())?;
        let id = storage.commit_base(base).await.expect("append");
        assert_eq!(BaseId(2), id);
        assert_eq!(BaseId(2), storage.max_id());
        Ok(())
    }

    #[tokio::test]
    async fn bases_are_spread_over_two_level_subfolders() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut storage = FileStorage::new(dir.path())?;
        let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
        for _ in 0..300 {
            storage.commit_base(base.clone()).await.expect("append");
        }
        let bases_dir = dir.path().join(FileStorage::BASES_DIR);
        // Id 1 lives in <bases>/0000/0001/00000001.postcard.
        assert!(
            bases_dir
                .join("0000")
                .join("0001")
                .join("00000001.postcard")
                .exists()
        );
        // Id 256 crosses into the next level-1 folder.
        assert!(
            bases_dir
                .join("0001")
                .join("0000")
                .join("00000100.postcard")
                .exists()
        );
        Ok(())
    }

    #[tokio::test]
    async fn readonly_snapshot_freezes_max_id_and_root() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
        let mut storage = FileStorage::new(dir.path())?;
        let id = storage.commit_base(base.clone()).await.expect("append");
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage)
            .await
            .expect("one_kv");
        storage.commit_root(root.clone()).await.expect("write root");
        let view = storage.snapshot();

        // Writes after the snapshot are invisible to the view.
        storage.commit_base(base.clone()).await.expect("append");
        let new_root = two_kv(
            HashKey::new(7),
            TrieValue::U32(7),
            HashKey::new(8),
            TrieValue::U32(8),
            &mut storage,
        )
        .await
        .expect("two_kv");
        storage.commit_root(new_root).await.expect("write root");

        assert_eq!(BaseId(4), storage.max_id());
        assert_eq!(BaseId(2), view.max_id());
        assert_eq!(root, view.read_root());
        assert_eq!(base, view.read_base(id).await.expect("read"));
        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "beyond this snapshot's max_id")]
    async fn readonly_snapshot_panics_reading_beyond_max_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = Base::new_kv(HashKey::new(7), TrieValue::U32(7));
        let mut storage = FileStorage::new(dir.path()).expect("new storage");
        storage.commit_base(base.clone()).await.expect("append");
        let view = storage.snapshot();
        let new_id = storage.commit_base(base).await.expect("append");
        let _ = view.read_base(new_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "base id 1 is beyond this snapshot's max_id")]
    async fn read_panics_on_unwritten_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = FileStorage::new(dir.path()).expect("new storage");
        let _ = storage.read_base(BaseId(1)).await;
    }
}
