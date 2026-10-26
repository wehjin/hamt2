use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use iroh::{PublicKey, SecretKey};

use crate::_internal::buffer;

/// The filename used to store the node's secret key.
pub const KEYPAIR_FILE: &str = "keypair";

#[derive(Error, Debug)]
pub enum InitError {
    #[error("already initialized: {0}")]
    AlreadyInitialized(PathBuf),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("buffer init: {0}")]
    Buffer(#[source] anyhow::Error),
}

/// Initializes a new database at the specified path.
///
/// Creates the directory (and any missing parents), writes a freshly generated
/// [`SecretKey`] to `keypair` (hex-encoded, owner-only), and initializes the
/// iroh-backed state (an empty root in the blob store, marked in a doc).
///
/// The key is the node's identity, so it is never overwritten: calling `init`
/// on an already-initialized path returns [`InitError::AlreadyInitialized`].
///
/// Returns the node's [`PublicKey`] (its identity on the network).
pub async fn init(path: impl AsRef<Path>) -> Result<PublicKey, InitError> {
    let path = path.as_ref();
    let secret_key = write_keypair(path)?;
    buffer::init(path, &secret_key)
        .await
        .map_err(|e| InitError::Buffer(e.into()))?;
    Ok(secret_key.public())
}

/// Generates and persists the node's secret key, returning it.
fn write_keypair(path: &Path) -> Result<SecretKey, InitError> {
    fs::create_dir_all(path)?;

    let keypair_path = path.join(KEYPAIR_FILE);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = match options.open(&keypair_path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(InitError::AlreadyInitialized(path.to_path_buf()));
        }
        Err(err) => return Err(err.into()),
    };

    let secret_key = SecretKey::generate();
    let hex = data_encoding::HEXLOWER.encode(&secret_key.to_bytes());
    file.write_all(hex.as_bytes())?;

    Ok(secret_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn init_creates_keypair() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        let node_id = init(&path).await.unwrap();

        let hex = fs::read_to_string(path.join(KEYPAIR_FILE)).unwrap();
        assert_eq!(hex.len(), 64);

        // The returned node id must match the persisted key.
        let secret_key: SecretKey = hex.parse().unwrap();
        assert_eq!(secret_key.public(), node_id);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn init_keypair_is_private() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        init(&path).await.unwrap();

        let mode = fs::metadata(path.join(KEYPAIR_FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[tokio::test]
    async fn init_creates_missing_parents() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("c");

        init(&path).await.unwrap();

        assert!(path.join(KEYPAIR_FILE).exists());
    }

    #[tokio::test]
    async fn init_refuses_to_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        init(&path).await.unwrap();
        let before = fs::read_to_string(path.join(KEYPAIR_FILE)).unwrap();

        let err = init(&path).await.unwrap_err();
        assert!(matches!(err, InitError::AlreadyInitialized(_)));

        let after = fs::read_to_string(path.join(KEYPAIR_FILE)).unwrap();
        assert_eq!(before, after);
    }

    #[tokio::test]
    async fn init_creates_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("db");

        init(&path).await.unwrap();

        let hex = fs::read_to_string(path.join(buffer::NAMESPACE_FILE)).unwrap();
        assert_eq!(hex.len(), 64);
        let _: iroh_docs::NamespaceId = hex.parse().unwrap();
    }
}
