use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

use iroh::{PublicKey, SecretKey};

/// The filename used to store the node's secret key.
pub const KEYPAIR_FILE: &str = "keypair";

#[derive(Error, Debug)]
pub enum InitError {
    #[error("already initialized: {0}")]
    AlreadyInitialized(PathBuf),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Initializes a new database at the specified path.
///
/// Creates the directory (and any missing parents) and writes a freshly
/// generated [`SecretKey`] to `keypair`. The key is the node's identity, so
/// it is never overwritten: calling `init` on an already-initialized path
/// returns [`InitError::AlreadyInitialized`].
///
/// Returns the node's [`PublicKey`] (its identity on the network).
pub fn init(path: impl AsRef<Path>) -> Result<PublicKey, InitError> {
    let path = path.as_ref();
    fs::create_dir_all(path)?;

    let keypair_path = path.join(KEYPAIR_FILE);
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&keypair_path)
    {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(InitError::AlreadyInitialized(path.to_path_buf()));
        }
        Err(err) => return Err(err.into()),
    };

    let secret_key = SecretKey::generate();
    file.write_all(&secret_key.to_bytes())?;

    Ok(secret_key.public())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn init_creates_keypair() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        let node_id = init(&path).unwrap();

        let bytes = fs::read(path.join(KEYPAIR_FILE)).unwrap();
        assert_eq!(bytes.len(), 32);

        // The returned node id must match the persisted key.
        let key: [u8; 32] = bytes.try_into().unwrap();
        assert_eq!(SecretKey::from_bytes(&key).public(), node_id);
    }

    #[test]
    fn init_creates_missing_parents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("c");

        init(&path).unwrap();

        assert!(path.join(KEYPAIR_FILE).exists());
    }

    #[test]
    fn init_refuses_to_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        init(&path).unwrap();
        let before = fs::read(path.join(KEYPAIR_FILE)).unwrap();

        let err = init(&path).unwrap_err();
        assert!(matches!(err, InitError::AlreadyInitialized(_)));

        let after = fs::read(path.join(KEYPAIR_FILE)).unwrap();
        assert_eq!(before, after);
    }
}
