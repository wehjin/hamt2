use crate::QueryError;
use sky_types::storage::errors::{StorageReadError, StorageWriteError};
use sky_types::trie::TrieInsertError;

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),

    #[error("StorageReadError: {0}")]
    StorageReadError(#[from] StorageReadError),

    #[error("StorageWriteError: {0}")]
    StorageWriteError(#[from] StorageWriteError),

    #[error("TrieWriteError: {0}")]
    TrieWriteError(#[from] TrieInsertError),

    #[error("No space in value table")]
    NoSpaceInValueTable,
}
