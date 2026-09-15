use sky_types::trie::error::TrieQueryError;
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
