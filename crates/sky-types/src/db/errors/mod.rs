use crate::storage::error::{ReadStorageError, WriteStorageError};
use crate::trie::{TrieInsertError, TrieQueryError};

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Trie: {0}")]
    TrieQueryError(#[from] TrieQueryError),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),

    #[error("ReadStorageError: {0}")]
    ReadStorageError(#[from] ReadStorageError),

    #[error("WriteStorageError: {0}")]
    WriteStorageError(#[from] WriteStorageError),

    #[error("TrieInsertError: {0}")]
    TrieInsertError(#[from] TrieInsertError),

    #[error("No space in value table")]
    NoSpaceInValueTable,

    #[error("Disconnected: {0}")]
    Disconnected(#[source] anyhow::Error),

    #[error("Refused: {0}")]
    Refused(#[source] anyhow::Error),
}
