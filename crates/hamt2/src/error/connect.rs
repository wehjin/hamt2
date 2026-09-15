use crate::{QueryError, TransactError};
use sky_trie::TrieWriteError;
use sky_trie::prelude::{TrieStorageReadError, TrieStorageWriteError};

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("Transact: {0}")]
    Transact(#[from] TransactError),

    #[error("TrieStorageRead: {0}")]
    TrieStorageRead(#[from] TrieStorageReadError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] TrieStorageWriteError),

    #[error("Trie: {0}")]
    Trie(#[from] TrieWriteError),

    #[error("NoSpaceInValueTable")]
    NoSpaceInValueTable,
}