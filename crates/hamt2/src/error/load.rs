use crate::db::Attr;
use crate::trie::base_storage::errors::BaseStorageReadError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Query error: {0:?}")]
    QueryError(#[from] crate::QueryError),

    #[error("Base storage read error: {0}")]
    BaseStorageRead(#[from] BaseStorageReadError),

    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
