use thiserror::Error;
use crate::space::table::TablePos;
use crate::space::{TableAddr, ValueAddr};
use crate::space::value::Val;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Invalid table addr {0}")]
    InvalidTableAddr(TableAddr),

    #[error("TablePos {0} with offset {1} exceeded the segment's length {2}")]
    TablePosWithOffsetExceedsSegmentLen(TablePos, usize, usize),

    #[error("Invalid value addr {0}")]
    InvalidValueAddr(ValueAddr),

    #[error("Invalid val {0}")]
    InvalidVal(Val),
}