use crate::storage::WriteStorageError;
use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {}

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("write_storage: {0}")]
    WriteStorage(#[from] WriteStorageError),

    #[error("trie_query: {0}")]
    TrieQuery(#[from] TrieQueryError),
}
