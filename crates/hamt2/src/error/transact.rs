use crate::QueryError;
use crate::trie::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("TrieStorageRead: {0}")]
    TrieStorageRead(#[from] TrieStorageReadError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] TrieStorageWriteError),

    #[error("Trie: {0}")]
    Trie(#[from] TrieWriteError),

    #[error("NoSpaceInValueTable")]
    NoSpaceInValueTable,
}
