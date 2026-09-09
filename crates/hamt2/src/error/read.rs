use crate::space::core::value::Val;
use crate::space::TableAddr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Slot address {0} offset {1} out of bounds")]
    SlotAddressOutOfBounds(TableAddr, usize),

    #[error("No slot value at table addr {0} offset {1}")]
    NoSlotValueAtTableAddrOffset(TableAddr, usize),

    #[error("Invalid val {0}")]
    InvalidVal(Val),
}
