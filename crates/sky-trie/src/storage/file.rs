use sky_types::storage::error::{ReadStorageError, WriteStorageError};
use crate::storage::{ReadStorage, ReadWriteStorage};
use crate::types::slot_base::SlotBase;
use sky_types::trie::MapBase;
use sky_types::trie::SlotBaseId;
use std::future;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

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
/// Base id [`SlotBaseId::ZERO`] is the reserved empty base and is never written to disk.
///
/// Clones share the same inner state (`Arc<RwLock<Inner>>`), so appends are
/// serialized through a shared `max_id` counter and clones never assign
/// duplicate ids or write stale state back to disk.
#[derive(Debug, Clone)]
pub struct FileStorage {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    bases_dir: PathBuf,
    max_id_path: PathBuf,
    root_path: PathBuf,
    max_id: i32,
}

impl FileStorage {
    const BASES_DIR: &'static str = "bases";
    const MAX_ID_FILE: &'static str = "max_id";
    const ROOT_FILE: &'static str = "root";

    /// Creates a fresh empty storage in the given folder.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = path.as_ref();
        let inner = Inner {
            bases_dir: root.join(Self::BASES_DIR),
            max_id_path: root.join(Self::MAX_ID_FILE),
            root_path: root.join(Self::ROOT_FILE),
            max_id: 0,
        };
        std::fs::create_dir_all(&inner.bases_dir)?;
        inner.write_max_id()?;
        inner.write_root_with(&MapBase::empty()).map_err(|e| match e {
            WriteStorageError::Io(_, e) => e,
            WriteStorageError::Encode(_, e) => std::io::Error::new(ErrorKind::InvalidData, e),
        })?;
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    /// Opens an existing storage. A missing folder or max id file is treated
    /// as an empty storage.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = path.as_ref();
        let inner = Inner {
            bases_dir: root.join(Self::BASES_DIR),
            max_id_path: root.join(Self::MAX_ID_FILE),
            root_path: root.join(Self::ROOT_FILE),
            max_id: 0,
        };
        std::fs::create_dir_all(&inner.bases_dir)?;
        let max_id = match std::fs::read(&inner.max_id_path) {
            Ok(bytes) => postcard::from_bytes::<i32>(&bytes)
                .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?,
            Err(e) if e.kind() == ErrorKind::NotFound => 0,
            Err(e) => return Err(e),
        };
        let inner = Inner { max_id, ..inner };
        Ok(Self {
            inner: Arc::new(RwLock::new(inner)),
        })
    }
}

/// The two-level subfolder layout of a base file under a `bases` dir.
fn base_path(bases_dir: &Path, id: SlotBaseId) -> PathBuf {
    let level_1 = id.0 >> 8;
    let level_2 = id.0 & 0xff;
    bases_dir
        .join(format!("{:04x}", level_1))
        .join(format!("{:04x}", level_2))
        .join(format!("{:08x}.postcard", id.0))
}

impl Inner {
    fn write_max_id(&self) -> Result<(), std::io::Error> {
        let bytes = postcard::to_allocvec(&self.max_id)
            .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
        std::fs::write(&self.max_id_path, bytes)
    }

    fn base_path(&self, id: SlotBaseId) -> PathBuf {
        base_path(&self.bases_dir, id)
    }

    fn write_max_id_with(&self, id: SlotBaseId) -> Result<(), WriteStorageError> {
        let bytes = postcard::to_allocvec(&id.0).map_err(|e| WriteStorageError::Encode(id, e))?;
        std::fs::write(&self.max_id_path, bytes).map_err(|e| WriteStorageError::Io(id, e))
    }

    fn read_root(&self) -> Result<MapBase, ReadStorageError> {
        match std::fs::read(&self.root_path) {
            Ok(bytes) => {
                let root = postcard::from_bytes::<MapBase>(&bytes)
                    .map_err(|e| ReadStorageError::Decode(SlotBaseId::ZERO, e))?;
                Ok(root)
            }
            // A missing root file is treated as an empty root.
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(MapBase::empty()),
            Err(e) => Err(ReadStorageError::Io(SlotBaseId::ZERO, e)),
        }
    }

    fn write_root_with(&self, root: &MapBase) -> Result<(), WriteStorageError> {
        let bytes =
            postcard::to_allocvec(root).map_err(|e| WriteStorageError::Encode(SlotBaseId::ZERO, e))?;
        std::fs::write(&self.root_path, bytes).map_err(|e| WriteStorageError::Io(SlotBaseId::ZERO, e))
    }
}

impl ReadStorage for FileStorage {
    type Snapshot = FileReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        let inner = self.inner.read().expect("storage poisoned");
        FileReadStorage {
            bases_dir: inner.bases_dir.clone(),
            max_id: inner.max_id,
            root: inner.read_root().expect("read root"),
        }
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        let inner = self.inner.read().expect("storage poisoned");
        assert!(
            id.0 <= inner.max_id,
            "base id {id} has not been written"
        );
        let path = inner.base_path(id);
        let bytes = std::fs::read(&path).map_err(|e| ReadStorageError::Io(id, e))?;
        let base = postcard::from_bytes::<SlotBase>(&bytes)
            .map_err(|e| ReadStorageError::Decode(id, e))?;
        Ok(base)
    }

    fn max_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId(inner.max_id)
    }

    async fn read_root(&self) -> Result<MapBase, ReadStorageError> {
        Ok(self.inner.read().expect("storage poisoned").read_root()?)
    }
}

/// A read-only, immutable view of a [`FileStorage`] taken at
/// [`ReadStorage::snapshot`] time.
///
/// `max_id` and `root` are captured into memory when the view is created, so
/// later appends or commits on the writer are invisible through it. Bases are
/// still read from disk: they are written once and never modified, so any id
/// at or below the captured `max_id` stays readable.
#[derive(Debug, Clone)]
pub struct FileReadStorage {
    bases_dir: PathBuf,
    max_id: i32,
    root: MapBase,
}

impl FileReadStorage {
    fn base_path(&self, id: SlotBaseId) -> PathBuf {
        base_path(&self.bases_dir, id)
    }
}

impl ReadStorage for FileReadStorage {
    type Snapshot = FileReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        assert!(
            id.0 <= self.max_id,
            "base id {id} is beyond this snapshot's max_id"
        );
        let path = self.base_path(id);
        let bytes = std::fs::read(&path).map_err(|e| ReadStorageError::Io(id, e))?;
        let base = postcard::from_bytes::<SlotBase>(&bytes)
            .map_err(|e| ReadStorageError::Decode(id, e))?;
        Ok(base)
    }

    fn max_id(&self) -> SlotBaseId {
        SlotBaseId(self.max_id)
    }

    async fn read_root(&self) -> Result<MapBase, ReadStorageError> {
        Ok(self.root.clone())
    }
}

impl ReadWriteStorage for FileStorage {
    fn next_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId(inner.max_id + 1)
    }

    fn append(
        &mut self,
        base: &SlotBase,
    ) -> impl Future<Output = Result<SlotBaseId, WriteStorageError>> {
        let mut inner = self.inner.write().expect("storage poisoned");
        let id = SlotBaseId(inner.max_id + 1);
        let bytes = match postcard::to_allocvec(base) {
            Ok(bytes) => bytes,
            Err(e) => return future::ready(Err(WriteStorageError::Encode(id, e))),
        };
        let path = inner.base_path(id);
        if let Err(e) = std::fs::create_dir_all(path.parent().expect("base path has parent")) {
            return future::ready(Err(WriteStorageError::Io(id, e)));
        }
        if let Err(e) = std::fs::write(&path, bytes) {
            return future::ready(Err(WriteStorageError::Io(id, e)));
        }
        if let Err(e) = inner.write_max_id_with(id) {
            return future::ready(Err(e));
        }
        inner.max_id = id.0;
        future::ready(Ok(id))
    }

    fn write_root(&mut self, root: MapBase) -> impl Future<Output = Result<(), WriteStorageError>> {
        match self
            .inner
            .read()
            .expect("storage poisoned")
            .write_root_with(&root)
        {
            Ok(()) => future::ready(Ok(())),
            Err(e) => future::ready(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crate_services::map_base::{one_kv, two_kv};
    use crate::types::HashKey;
    use sky_types::trie::TrieValue;

    #[tokio::test]
    async fn empty_storage_max_id_is_zero() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let storage = FileStorage::new(dir.path())?;
        assert_eq!(SlotBaseId::ZERO, storage.max_id());
        assert_eq!(SlotBaseId(1), storage.next_id());
        assert_eq!(
            SlotBase::new(),
            storage.read(SlotBaseId::ZERO).await.expect("read")
        );
        Ok(())
    }

    #[tokio::test]
    async fn append_and_reload_works() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let bases = (1..=10)
            .map(|i| SlotBase::new_kv(HashKey::new(i), TrieValue::U32(i as u32)))
            .collect::<Vec<_>>();
        {
            let mut storage = FileStorage::new(dir.path())?;
            for base in &bases {
                storage.append(base).await.expect("append");
            }
            assert_eq!(SlotBaseId(10), storage.max_id());
            assert_eq!(SlotBaseId(11), storage.next_id());
            for (i, base) in bases.iter().enumerate() {
                let id = SlotBaseId(i as i32 + 1);
                assert_eq!(base, &storage.read(id).await.expect("read"));
            }
        }
        let storage = FileStorage::load(dir.path())?;
        assert_eq!(SlotBaseId(10), storage.max_id());
        for (i, base) in bases.iter().enumerate() {
            let id = SlotBaseId(i as i32 + 1);
            assert_eq!(base, &storage.read(id).await.expect("read"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn root_round_trip_works() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let root;
        {
            let mut storage = FileStorage::new(dir.path())?;
            assert_eq!(
                MapBase::empty(),
                storage.read_root().await.expect("read root")
            );
            root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage).await;
            storage.write_root(root.clone()).await.expect("write root");
        }
        let storage = FileStorage::load(dir.path())?;
        assert_eq!(root, storage.read_root().await.expect("read root"));
        Ok(())
    }

    #[tokio::test]
    async fn appending_continues_after_reload() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        {
            let mut storage = FileStorage::new(dir.path())?;
            storage.append(&base).await.expect("append");
        }
        let mut storage = FileStorage::load(dir.path())?;
        let id = storage.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(2), id);
        assert_eq!(SlotBaseId(2), storage.max_id());
        Ok(())
    }

    #[tokio::test]
    async fn clones_share_the_same_id_counter() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut storage = FileStorage::new(dir.path())?;
        let mut clone = storage.clone();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = clone.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(1), id0);
        assert_eq!(SlotBaseId(2), id1);
        assert_eq!(SlotBaseId(2), storage.max_id());
        assert_eq!(SlotBaseId(2), clone.max_id());
        Ok(())
    }

    #[tokio::test]
    async fn bases_are_spread_over_two_level_subfolders() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut storage = FileStorage::new(dir.path())?;
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        for _ in 0..300 {
            storage.append(&base).await.expect("append");
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
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let mut storage = FileStorage::new(dir.path())?;
        let id = storage.append(&base).await.expect("append");
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage).await;
        storage.write_root(root.clone()).await.expect("write root");
        let view = storage.snapshot();

        // Writes after the snapshot are invisible to the view.
        storage.append(&base).await.expect("append");
        let new_root = two_kv(
            HashKey::new(7),
            TrieValue::U32(7),
            HashKey::new(8),
            TrieValue::U32(8),
            &mut storage,
        )
        .await;
        storage.write_root(new_root).await.expect("write root");

        assert_eq!(SlotBaseId(4), storage.max_id());
        assert_eq!(SlotBaseId(2), view.max_id());
        assert_eq!(root, view.read_root().await.expect("read root"));
        assert_eq!(base, view.read(id).await.expect("read"));
        Ok(())
    }

    #[tokio::test]
    #[should_panic(expected = "beyond this snapshot's max_id")]
    async fn readonly_snapshot_panics_reading_beyond_max_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let mut storage = FileStorage::new(dir.path()).expect("new storage");
        storage.append(&base).await.expect("append");
        let view = storage.snapshot();
        let new_id = storage.append(&base).await.expect("append");
        let _ = view.read(new_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "has not been written")]
    async fn read_panics_on_unwritten_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = FileStorage::new(dir.path()).expect("new storage");
        let _ = storage.read(SlotBaseId(1)).await;
    }
}
