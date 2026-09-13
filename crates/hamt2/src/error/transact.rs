use crate::QueryError;
use crate::trie::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("TrieStorageRead: {0}")]
    TrieStorageRead(#[from] TrieStorageReadError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] TrieStorageWriteError),

    #[error("Trie: {0}")]
    Trie(#[from] TrieWriteError),

    #[error("HighBitInValue: {0}")]
    HighBitInValue(u32),

    #[error("NoSpaceInValueTable")]
    NoSpaceInValueTable,
}
