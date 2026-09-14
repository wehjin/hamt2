use thiserror::Error;

/// An error from a trie query operation.
#[derive(Debug, Error)]
pub enum TrieQueryError {
    #[error("An unexpected error occurred: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
