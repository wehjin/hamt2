use crate::storage::WriteStorageError;
use thiserror::Error;

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("write_storage: {0}")]
    WriteStorage(#[from] WriteStorageError),
}
