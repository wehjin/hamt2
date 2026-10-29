#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("Transact: {0}")]
    Transact(#[from] TransactError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("log: {0}")]
    Log(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("QueryError: {0}")]
    QueryError(#[from] QueryError),

    #[error("edit trie: {0}")]
    Rewind(#[source] anyhow::Error),

    #[error("No space in value table")]
    NoSpaceInValueTable,

    #[error("Disconnected: {0}")]
    Disconnected(#[source] anyhow::Error),

    #[error("Refused: {0}")]
    Refused(#[source] anyhow::Error),
}
