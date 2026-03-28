use crate::db::attr::Attr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Query error: {0:?}")]
    QueryError(#[from] crate::QueryError),

    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
