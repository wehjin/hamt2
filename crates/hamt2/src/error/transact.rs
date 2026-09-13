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

    #[error("BaseStorageRead: {0}")]
    BaseStorageRead(#[from] TrieStorageReadError),

    #[error("BaseStorageWrite: {0}")]
    BaseStorageWrite(#[from] TrieStorageWriteError),

    #[error("HighBitInValue: {0}")]
    HighBitInValue(u32),

    #[error("ExpectedMapBaseAtKey")]
    ExpectedMapBaseAtKey,

    #[error("NoSpaceInValueTable")]
    NoSpaceInValueTable,
}
