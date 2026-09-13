use crate::types::slot_base_id::SlotBaseId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TrieStorageReadError {
    #[error("base id {0} has not been written")]
    NotFound(SlotBaseId),
    #[error("failed to read base id {0} from disk: {1}")]
    Io(SlotBaseId, #[source] std::io::Error),
    #[error("failed to decode base id {0}: {1}")]
    Decode(SlotBaseId, #[source] postcard::Error),
}

#[derive(Debug, Error)]
pub enum TrieStorageWriteError {
    #[error("failed to write base id {0} to disk: {1}")]
    Io(SlotBaseId, #[source] std::io::Error),
    #[error("failed to encode base id {0}: {1}")]
    Encode(SlotBaseId, #[source] postcard::Error),
}
