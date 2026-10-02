use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteStorageError {
    #[error("failed to write {0} to disk: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("failed to encode {0}: {1}")]
    Encode(String, #[source] postcard::Error),
    #[error("write storage failed: {0}")]
    Anyhow(#[from] anyhow::Error),
}
