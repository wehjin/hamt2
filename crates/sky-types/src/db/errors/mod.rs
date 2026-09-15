use crate::storage::error::{StorageReadError, StorageWriteError};
use crate::trie::{TrieInsertError, TrieQueryError};
use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Trie: {0}")]
    TrieQueryError(#[from] TrieQueryError),

    #[error("SerdeError: {0}")]
    SerdeError(String),
}

impl serde::de::Error for QueryError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        QueryError::SerdeError(msg.to_string())
    }
}

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
