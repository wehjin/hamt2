use crate::iroh_db::client::keys;

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Io: {0}")]
    Io(#[from] std::io::Error),

    #[error("Utf8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Key: {0}")]
    Key(#[from] keys::Error),

    #[error("SpaceReadError: {0}")]
    SpaceReadError(#[from] crate::hamt::space::ReadError),

    #[error("NotAValue: {0}")]
    NotAValue(u32),

    #[error("InvalidSlotType")]
    InvalidSlotType,

    #[error("MismatchedKeys: {0} != {1}")]
    MismatchedKeys(i32, i32),

    #[error("BaseIndexOutOfBounds: {0}")]
    BaseIndexOutOfBounds(usize),

    #[error("ExpectedMapBaseAtKey: {0}")]
    NoSubtrieAtKeyIndex(usize),

    #[error("NoRootInReader")]
    NoRootInReader,
}
