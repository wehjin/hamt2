use crate::db::Attr;
use crate::trie::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Query error: {0:?}")]
    QueryError(#[from] crate::QueryError),

    #[error("Trie storage read: {0}")]
    TrieStorageRead(#[from] TrieStorageReadError),

    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
