use std::fmt::Display;

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

    #[error("SpaceReadError: {0}")]
    SpaceReadError(#[from] crate::error::ReadError),

    #[error("NotAValue: {0}")]
    NotAValue(u32),

    #[error("InvalidSlotType")]
    InvalidSlotType,

    #[error("MismatchedKeys: {0} != {1}")]
    MismatchedKeys(i32, i32),

    #[error("BaseIndexOutOfBounds: {0}")]
    BaseIndexOutOfBounds(usize),

    #[error("NoRootInReader")]
    NoRootInReader,

    #[error("SerdeError: {0}")]
    SerdeError(String),
}

impl serde::de::Error for QueryError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        QueryError::SerdeError(msg.to_string())
    }
}
