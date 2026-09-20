use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum StoreReadError {
    #[error("not available")]
    NotAvailable,
}

#[derive(Error, Debug, Clone)]
pub enum StoreEditError {
    #[error("no slots available")]
    NoSlotsAvailable,
}
