#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("log: {0}")]
    Log(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("edit trie: {0}")]
    Rewind(#[source] anyhow::Error),

    #[error("No space in value table")]
    NoSpaceInValueTable,
}
