use crate::db::Attr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("Duplicate attr: {0}")]
    DuplicateAttr(Attr),
}
