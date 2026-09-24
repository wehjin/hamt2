use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{Base, BaseId, MapBase};
use std::path::{Path, PathBuf};
use tokio::fs;

pub const BASES_DIR: &'static str = "bases";

pub async fn init_bases_dir_with_empty_base(
    frame_dir: impl AsRef<Path>,
) -> Result<PathBuf, WriteStorageError> {
    let bases_dir = bases_dir(frame_dir);
    assert_eq!(bases_dir.exists(), false, "bases dir already exists");
    fs::create_dir_all(&bases_dir)
        .await
        .expect("create bases dir");
    write_base(&bases_dir, BaseId::ZERO, Base::empty()).await?;
    Ok(bases_dir)
}

pub fn bases_dir(frame_dir: impl AsRef<Path>) -> PathBuf {
    frame_dir.as_ref().to_path_buf().join(BASES_DIR)
}

/// The two-level subfolder layout of a base file under a `bases` dir.
pub fn base_path(bases_dir: impl AsRef<Path>, id: BaseId) -> PathBuf {
    let bases_dir = bases_dir.as_ref();
    let level_1 = id.0 >> 8;
    let level_2 = id.0 & 0xff;
    bases_dir
        .join(format!("{:04x}", level_1))
        .join(format!("{:04x}", level_2))
        .join(format!("{:08x}.postcard", id.0))
}

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
pub async fn read_base(bases_dir: impl AsRef<Path>, id: BaseId) -> Result<Base, ReadStorageError> {
    let path = base_path(bases_dir, id);
    let Ok(true) = path.try_exists() else {
        return Ok(Base::empty());
    };
    let bytes = fs::read(&path)
        .await
        .map_err(|e| ReadStorageError::Io(format!("{id:?}"), e))?;
    postcard::from_bytes::<Base>(&bytes).map_err(|e| ReadStorageError::Decode(format!("{id:?}"), e))
}

pub async fn write_base(
    bases_dir: impl AsRef<Path>,
    id: BaseId,
    base: Base,
) -> Result<(), WriteStorageError> {
    let bytes = match postcard::to_allocvec(&base) {
        Ok(bytes) => bytes,
        Err(e) => return Err(WriteStorageError::Encode(format!("{id:?}"), e)),
    };
    let path = base_path(bases_dir, id);
    let base_parent_dir = path.parent().expect("base path has parent");
    fs::create_dir_all(base_parent_dir).await.map_err(|e| {
        let name = base_parent_dir.display().to_string();
        WriteStorageError::Io(format!("({id:?} parent)[{name}]"), e)
    })?;
    fs::write(&path, bytes)
        .await
        .map_err(|e| WriteStorageError::Io(format!("{id:?}"), e))
}

pub async fn read_max_id_file(path: &PathBuf) -> Result<BaseId, ReadStorageError> {
    let bytes = fs::read(&path)
        .await
        .map_err(|e| ReadStorageError::Io("max_id".to_string(), e))?;
    postcard::from_bytes::<i32>(&bytes)
        .map(|i32| BaseId(i32))
        .map_err(|e| ReadStorageError::Decode("max_id".to_string(), e))
}

pub async fn write_max_id_file(path: &Path, id: BaseId) -> Result<(), WriteStorageError> {
    let bytes = postcard::to_allocvec(&id.0)
        .map_err(|e| WriteStorageError::Encode("max_id".to_string(), e))?;
    fs::write(path, bytes)
        .await
        .map_err(|e| WriteStorageError::Io("max_id".to_string(), e))
}

pub async fn read_root_file(path: &Path) -> Result<MapBase, ReadStorageError> {
    let bytes = fs::read(path)
        .await
        .map_err(|e| ReadStorageError::Io("root".to_string(), e))?;
    postcard::from_bytes::<MapBase>(&bytes)
        .map_err(|e| ReadStorageError::Decode("root".to_string(), e))
}

pub async fn write_root_file(path: &Path, root: &MapBase) -> Result<(), WriteStorageError> {
    let bytes = postcard::to_allocvec(root)
        .map_err(|e| WriteStorageError::Encode("root".to_string(), e))?;
    fs::write(path, bytes)
        .await
        .map_err(|e| WriteStorageError::Io("root".to_string(), e))
}
