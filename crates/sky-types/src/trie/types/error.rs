use crate::storage::{ReadStorageError, WriteStorageError};
use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {
    #[error("read_storage: {0}")]
    ReadStorage(#[from] ReadStorageError),

    #[error("An unexpected error occurred: {0}")]
    SystemError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("write_storage: {0}")]
    WriteStorage(#[from] WriteStorageError),

    #[error("trie_query: {0}")]
    TrieQuery(#[from] TrieQueryError),

    #[error("read_storage: {0}")]
    ReadStorage(#[from] ReadStorageError),
}
