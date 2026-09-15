use crate::{QueryError, TransactError};
use sky_trie::prelude::{StorageReadError, StorageWriteError};

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("Transact: {0}")]
    Transact(#[from] TransactError),

    #[error("TrieStorageRead: {0}")]
    TrieStorageRead(#[from] StorageReadError),

    #[error("TrieStorageWrite: {0}")]
    TrieStorageWrite(#[from] StorageWriteError),
}