use crate::storage::{ReadStorageError, WriteStorageError};
use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {
    #[error("An unexpected error occurred: {0}")]
    SystemError(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("read_storage: {0}")]
    ReadStorage(#[from] ReadStorageError),
}

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("trie_query: {0}")]
    TrieQuery(#[from] TrieQueryError),

    #[error("write_storage: {0}")]
    WriteStorage(#[from] WriteStorageError),
}
