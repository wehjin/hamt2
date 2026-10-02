use sky_types::db::{QueryError, TransactError};
use sky_types::storage::error::WriteStorageError;

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("Transact: {0}")]
    Transact(#[from] TransactError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] WriteStorageError),

    #[error("trie edit: {0}")]
    TrieEdit(#[source] anyhow::Error),
}
