use crate::trie_storage::errors::{TrieStorageReadError, TrieStorageWriteError};
use crate::trie_storage::{ReadTrieStorage, ReadWriteTrieStorage};
use crate::types::slot_base::SlotBase;
use sky_types::trie::MapBase
;
use sky_types::trie::SlotBaseId
;
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
/// Base id 0 is the reserved empty base and is never written to disk.
///
/// Clones share the same inner state (`Arc<RwLock<Inner>>`), so appends are
/// serialized through a shared `max_id` counter and clones never assign
/// duplicate ids or write stale state back to disk.
#[derive(Debug, Clone)]
pub struct FileTrieStorage {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Debug)]
struct Inner {
    bases_dir: PathBuf,
    max_id_path: PathBuf,
    root_path: PathBuf,
    max_id: i32,
}

impl FileTrieStorage {
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

    fn write_max_id_with(&self, id: SlotBaseId) -> Result<(), TrieStorageWriteError> {
        let bytes =
            postcard::to_allocvec(&id.0).map_err(|e| TrieStorageWriteError::Encode(id, e))?;
        std::fs::write(&self.max_id_path, bytes).map_err(|e| TrieStorageWriteError::Io(id, e))
    }

    fn read_root(&self) -> Result<Option<MapBase>, TrieStorageReadError> {
        match std::fs::read(&self.root_path) {
            Ok(bytes) => {
                let root = postcard::from_bytes::<MapBase>(&bytes)
                    .map_err(|e| TrieStorageReadError::Decode(SlotBaseId(0), e))?;
                Ok(Some(root))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(TrieStorageReadError::Io(SlotBaseId(0), e)),
        }
    }

    fn write_root_with(&self, root: &MapBase) -> Result<(), TrieStorageWriteError> {
        let bytes = postcard::to_allocvec(root)
            .map_err(|e| TrieStorageWriteError::Encode(SlotBaseId(0), e))?;
        std::fs::write(&self.root_path, bytes)
            .map_err(|e| TrieStorageWriteError::Io(SlotBaseId(0), e))
    }
}

impl ReadTrieStorage for FileTrieStorage {
    type Snapshot = FileReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        let inner = self.inner.read().expect("storage poisoned");
        FileReadStorage {
            bases_dir: inner.bases_dir.clone(),
            max_id: inner.max_id,
            root: inner.read_root().expect("read root"),
        }
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, TrieStorageReadError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        let inner = self.inner.read().expect("storage poisoned");
        if id.0 > inner.max_id {
            return Err(TrieStorageReadError::NotFound(id));
        }
        let path = inner.base_path(id);
        let bytes = std::fs::read(&path).map_err(|e| TrieStorageReadError::Io(id, e))?;
        let base = postcard::from_bytes::<SlotBase>(&bytes)
            .map_err(|e| TrieStorageReadError::Decode(id, e))?;
        Ok(base)
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        let inner = self.inner.read().expect("storage poisoned");
        if inner.max_id == 0 {
            None
        } else {
            Some(SlotBaseId(inner.max_id))
        }
    }

    async fn read_root(&self) -> Result<Option<MapBase>, TrieStorageReadError> {
        Ok(self.inner.read().expect("storage poisoned").read_root()?)
    }
}

/// A read-only, immutable view of a [`FileTrieStorage`] taken at
/// [`ReadTrieStorage::snapshot`] time.
///
/// `max_id` and `root` are captured into memory when the view is created, so
/// later appends or commits on the writer are invisible through it. Bases are
/// still read from disk: they are written once and never modified, so any id
/// at or below the captured `max_id` stays readable.
#[derive(Debug, Clone)]
pub struct FileReadStorage {
    bases_dir: PathBuf,
    max_id: i32,
    root: Option<MapBase>,
}

impl FileReadStorage {
    fn base_path(&self, id: SlotBaseId) -> PathBuf {
        base_path(&self.bases_dir, id)
    }
}

impl ReadTrieStorage for FileReadStorage {
    type Snapshot = FileReadStorage;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, TrieStorageReadError> {
        if id.0 == 0 {
            return Ok(SlotBase::new());
        }
        if id.0 > self.max_id {
            return Err(TrieStorageReadError::NotFound(id));
        }
        let path = self.base_path(id);
        let bytes = std::fs::read(&path).map_err(|e| TrieStorageReadError::Io(id, e))?;
        let base = postcard::from_bytes::<SlotBase>(&bytes)
            .map_err(|e| TrieStorageReadError::Decode(id, e))?;
        Ok(base)
    }

    fn max_id(&self) -> Option<SlotBaseId> {
        if self.max_id == 0 {
            None
        } else {
            Some(SlotBaseId(self.max_id))
        }
    }

    async fn read_root(&self) -> Result<Option<MapBase>, TrieStorageReadError> {
        Ok(self.root.clone())
    }
}

impl ReadWriteTrieStorage for FileTrieStorage {
    fn next_id(&self) -> SlotBaseId {
        let inner = self.inner.read().expect("storage poisoned");
        SlotBaseId(inner.max_id + 1)
    }

    fn append(
        &mut self,
        base: &SlotBase,
    ) -> impl Future<Output = Result<SlotBaseId, TrieStorageWriteError>> {
        let mut inner = self.inner.write().expect("storage poisoned");
        let id = SlotBaseId(inner.max_id + 1);
        let bytes = match postcard::to_allocvec(base) {
            Ok(bytes) => bytes,
            Err(e) => return future::ready(Err(TrieStorageWriteError::Encode(id, e))),
        };
        let path = inner.base_path(id);
        if let Err(e) = std::fs::create_dir_all(path.parent().expect("base path has parent")) {
            return future::ready(Err(TrieStorageWriteError::Io(id, e)));
        }
        if let Err(e) = std::fs::write(&path, bytes) {
            return future::ready(Err(TrieStorageWriteError::Io(id, e)));
        }
        if let Err(e) = inner.write_max_id_with(id) {
            return future::ready(Err(e));
        }
        inner.max_id = id.0;
        future::ready(Ok(id))
    }

    fn write_root(
        &mut self,
        root: MapBase,
    ) -> impl Future<Output = Result<(), TrieStorageWriteError>> {
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
    use sky_types::trie::TrieValue
;

    #[tokio::test]
    async fn empty_storage_has_no_ids() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let storage = FileTrieStorage::new(dir.path())?;
        assert_eq!(None, storage.max_id());
        assert_eq!(SlotBaseId(1), storage.next_id());
        assert_eq!(
            SlotBase::new(),
            storage.read(SlotBaseId(0)).await.expect("read")
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
            let mut storage = FileTrieStorage::new(dir.path())?;
            for base in &bases {
                storage.append(base).await.expect("append");
            }
            assert_eq!(Some(SlotBaseId(10)), storage.max_id());
            assert_eq!(SlotBaseId(11), storage.next_id());
            for (i, base) in bases.iter().enumerate() {
                let id = SlotBaseId(i as i32 + 1);
                assert_eq!(base, &storage.read(id).await.expect("read"));
            }
        }
        let storage = FileTrieStorage::load(dir.path())?;
        assert_eq!(Some(SlotBaseId(10)), storage.max_id());
        for (i, base) in bases.iter().enumerate() {
            let id = SlotBaseId(i as i32 + 1);
            assert_eq!(base, &storage.read(id).await.expect("read"));
        }
        Ok(())
    }

    #[tokio::test]
    async fn root_round_trip_works() -> anyhow::Result<()> {
        use sky_types::trie::MapBase
;
        let dir = tempfile::tempdir()?;
        {
            let mut storage = FileTrieStorage::new(dir.path())?;
            assert_eq!(None, storage.read_root().await.expect("read root"));
            storage
                .write_root(MapBase::empty())
                .await
                .expect("write root");
        }
        let storage = FileTrieStorage::load(dir.path())?;
        assert_eq!(
            Some(MapBase::empty()),
            storage.read_root().await.expect("read root")
        );
        Ok(())
    }

    #[tokio::test]
    async fn appending_continues_after_reload() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        {
            let mut storage = FileTrieStorage::new(dir.path())?;
            storage.append(&base).await.expect("append");
        }
        let mut storage = FileTrieStorage::load(dir.path())?;
        let id = storage.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(2), id);
        assert_eq!(Some(SlotBaseId(2)), storage.max_id());
        Ok(())
    }

    #[tokio::test]
    async fn clones_share_the_same_id_counter() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut storage = FileTrieStorage::new(dir.path())?;
        let mut clone = storage.clone();
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = clone.append(&base).await.expect("append");
        assert_eq!(SlotBaseId(1), id0);
        assert_eq!(SlotBaseId(2), id1);
        assert_eq!(Some(SlotBaseId(2)), storage.max_id());
        assert_eq!(Some(SlotBaseId(2)), clone.max_id());
        Ok(())
    }

    #[tokio::test]
    async fn bases_are_spread_over_two_level_subfolders() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let mut storage = FileTrieStorage::new(dir.path())?;
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        for _ in 0..300 {
            storage.append(&base).await.expect("append");
        }
        let bases_dir = dir.path().join(FileTrieStorage::BASES_DIR);
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
        use crate::trie_storage::errors::TrieStorageReadError;
        let dir = tempfile::tempdir()?;
        let base = SlotBase::new_kv(HashKey::new(7), TrieValue::U32(7));
        let mut storage = FileTrieStorage::new(dir.path())?;
        let id = storage.append(&base).await.expect("append");
        let root = one_kv(HashKey::new(7), TrieValue::U32(7), &mut storage).await;
        storage.write_root(root.clone()).await.expect("write root");
        let view = storage.snapshot();

        // Writes after the snapshot are invisible to the view.
        let new_id = storage.append(&base).await.expect("append");
        let new_root = two_kv(
            HashKey::new(7),
            TrieValue::U32(7),
            HashKey::new(8),
            TrieValue::U32(8),
            &mut storage,
        )
        .await;
        storage.write_root(new_root).await.expect("write root");

        assert_eq!(Some(SlotBaseId(4)), storage.max_id());
        assert_eq!(Some(SlotBaseId(2)), view.max_id());
        assert_eq!(Some(root), view.read_root().await.expect("read root"));
        assert_eq!(base, view.read(id).await.expect("read"));
        assert!(matches!(
            view.read(new_id).await,
            Err(TrieStorageReadError::NotFound(rid)) if rid == new_id
        ));
        Ok(())
    }
}
