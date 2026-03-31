use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Red database error: {0:?}")]
    RedDatabase(#[from] redb::DatabaseError),

    #[error("Red transaction error: {0:?}")]
    RedTransaction(#[from] redb::TransactionError),

    #[error("Red table error: {0:?}")]
    RedTable(#[from] redb::TableError),

    #[error("Red storage error: {0:?}")]
    RedStorage(#[from] redb::StorageError),

    #[error("Red commit error: {0:?}")]
    RedCommit(#[from] redb::CommitError),

    #[error("Postcard error: {0:?}")]
    Postcard(#[from] postcard::Error),

    #[error("Anyhow error: {0:?}")]
    Anyhow(#[from] anyhow::Error),
}
