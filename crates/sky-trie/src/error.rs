use sky_types::trie::error::TrieQueryError;
use thiserror::Error;

/// An error from a trie mutation (insert) operation.
#[derive(Debug, Error)]
pub enum TrieInsertError {
    #[error("expected a map base at the key")]
    ExpectedMapBaseAtKey,
    #[error("query failed during mutation: {0}")]
    Query(#[from] TrieQueryError),
}
