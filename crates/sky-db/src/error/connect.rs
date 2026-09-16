use sky_types::db::{QueryError, TransactError};
use sky_types::storage::error::{ReadStorageError, WriteStorageError};

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("Transact: {0}")]
    Transact(#[from] TransactError),

    #[error("TrieStorageRead: {0}")]
    TrieStorageRead(#[from] ReadStorageError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] WriteStorageError),
}