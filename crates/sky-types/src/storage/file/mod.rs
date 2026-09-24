use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{BaseId, MapBase};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

mod edit;
mod read;

pub use edit::*;
pub use read::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{TrieEdit, TrieView};
    use crate::trie::map_base::{one_kv, two_kv};
    use crate::trie::{Base, BaseCommit, BaseRead, HashKey, TrieValue};

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
