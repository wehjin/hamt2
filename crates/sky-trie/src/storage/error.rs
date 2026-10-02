use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteStorageError {
    #[error("write storage failed: {0}")]
    Anyhow(#[from] anyhow::Error),
}
