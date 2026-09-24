use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{BaseId, MapBase};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// The two-level subfolder layout of a base file under a `bases` dir.
pub fn base_path(bases_dir: &Path, id: BaseId) -> PathBuf {
    let level_1 = id.0 >> 8;
    let level_2 = id.0 & 0xff;
    bases_dir
        .join(format!("{:04x}", level_1))
        .join(format!("{:04x}", level_2))
        .join(format!("{:08x}.postcard", id.0))
}

/// Writes the highest written base id to the `max_id` file.
pub fn write_max_id_file(path: &Path, id: BaseId) -> Result<(), WriteStorageError> {
    let bytes = postcard::to_allocvec(&id.0).map_err(|e| WriteStorageError::Encode(id, e))?;
    std::fs::write(path, bytes).map_err(|e| WriteStorageError::Io(id, e))
}

/// Reads the committed root from the `root` file. A missing root file is
/// treated as an empty root.
pub fn read_root_file(path: &Path) -> Result<MapBase, ReadStorageError> {
    match std::fs::read(path) {
        Ok(bytes) => postcard::from_bytes::<MapBase>(&bytes)
            .map_err(|e| ReadStorageError::Decode(BaseId::ZERO, e)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(MapBase::empty()),
        Err(e) => Err(ReadStorageError::Io(BaseId::ZERO, e)),
    }
}

/// Writes the committed root to the `root` file.
pub fn write_root_file(path: &Path, root: &MapBase) -> Result<(), WriteStorageError> {
    let bytes =
        postcard::to_allocvec(root).map_err(|e| WriteStorageError::Encode(BaseId::ZERO, e))?;
    std::fs::write(path, bytes).map_err(|e| WriteStorageError::Io(BaseId::ZERO, e))
}

pub fn write_to_io(e: WriteStorageError) -> std::io::Error {
    match e {
        WriteStorageError::Io(_, e) => e,
        WriteStorageError::Encode(_, e) => std::io::Error::new(ErrorKind::InvalidData, e),
    }
}

pub fn read_to_io(e: ReadStorageError) -> std::io::Error {
    match e {
        ReadStorageError::Io(_, e) => e,
        ReadStorageError::Decode(_, e) => std::io::Error::new(ErrorKind::InvalidData, e),
    }
}