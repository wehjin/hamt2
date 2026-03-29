use crate::space::core::value::Val;
use crate::space::TableAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Invalid table addr {0}")]
    InvalidTableAddr(TableAddr),

    #[error("Invalid val {0}")]
    InvalidVal(Val),
}
