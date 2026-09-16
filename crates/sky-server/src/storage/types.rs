use sky_db::ConnectError;
use sky_trie::types::StorageHead;
use sky_types::db::TransactError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageServiceError {
    #[error("Connection error: {0}")]
    ConnectError(#[from] ConnectError),

    #[error("Transport error: {0}")]
    TransactError(#[from] TransactError),
}

/// These are messages received from the storage service after connection.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StorageBroadcastEvent {
    TransactFailed(String),
    NewHead(StorageHead),
}