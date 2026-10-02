use crate::trie_storage::error::WriteStorageError;

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

use crate::trie::{TrieInsertError, TrieQueryError};

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("TrieQueryError: {0}")]
    TrieQueryError(#[from] TrieQueryError),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),

    #[error("WriteStorageError: {0}")]
    WriteStorageError(#[from] WriteStorageError),

    #[error("TrieQueryError: {0}")]
    TrieQueryError(#[from] TrieQueryError),

    #[error("TrieInsertError: {0}")]
    TrieInsertError(#[from] TrieInsertError),

    #[error("edit trie: {0}")]
    Rewind(#[source] anyhow::Error),

    #[error("No space in value table")]
    NoSpaceInValueTable,

    #[error("Disconnected: {0}")]
    Disconnected(#[source] anyhow::Error),

    #[error("Refused: {0}")]
    Refused(#[source] anyhow::Error),
}
