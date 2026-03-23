use crate::QueryError;
use crate::space;
use crate::space::seg::Seg;

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("HighBitInValue: {0}")]
    HighBitInValue(u32),

    #[error("InvalidSlotType")]
    InvalidSlotType,

    #[error("SlotOccupied")]
    SlotOccupied,

    #[error("SlotUnoccupied")]
    SlotEmpty,

    #[error("ExpectedMapBaseAtKey")]
    ExpectedMapBaseAtKey,

    #[error("SpaceReadError: {0}")]
    SpaceReadError(#[from] space::ReadError),

    #[error("Segment {0} already exists")]
    SegConflict(Seg),
}