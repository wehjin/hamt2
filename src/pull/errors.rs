use crate::db::Attr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Duplicate attr: {0}")]
    DuplicateAttr(Attr),
}

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("QueryError: {0}")]
    Query(#[from] crate::QueryError),
}
