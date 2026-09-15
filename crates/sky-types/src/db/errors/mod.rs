use crate::storage::error::{StorageReadError, StorageWriteError};
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

    #[error("StorageReadError: {0}")]
    StorageReadError(#[from] StorageReadError),

    #[error("StorageWriteError: {0}")]
    StorageWriteError(#[from] StorageWriteError),

    #[error("TrieInsertError: {0}")]
    TrieInsertError(#[from] TrieInsertError),

    #[error("No space in value table")]
    NoSpaceInValueTable,
}
