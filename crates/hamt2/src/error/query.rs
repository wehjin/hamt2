use std::fmt::Display;

use crate::trie::prelude::*;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Io: {0}")]
    Io(#[from] std::io::Error),

    #[error("Utf8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Trie: {0}")]
    Trie(#[from] TrieQueryError),

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
