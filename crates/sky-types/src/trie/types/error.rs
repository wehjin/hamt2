use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {
    #[error("An unexpected error occurred: {0}")]
    SystemError(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("query failed during mutation: {0}")]
    Query(#[from] TrieQueryError),
}
