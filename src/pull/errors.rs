use crate::db::Attr;
use serde::ser;
use std::fmt::Display;
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

#[derive(Error, Debug)]
pub enum PushError {
    #[error("Serde {0}")]
    CustomSerde(String),
}

impl ser::Error for PushError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::CustomSerde(msg.to_string())
    }
}
