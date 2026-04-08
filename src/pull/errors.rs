use crate::db::attr::Attr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Duplicate attr: {0}")]
    DuplicateAttr(Attr),
}
