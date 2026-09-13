use crate::trie_storage::errors::TrieStorageReadError;
use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {
    #[error("failed to read base storage: {0}")]
    Storage(#[from] TrieStorageReadError),
}

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieWriteError {
    #[error("expected a map base at the key")]
    ExpectedMapBaseAtKey,
    #[error("query failed during mutation: {0}")]
    Query(#[from] TrieQueryError),
}
