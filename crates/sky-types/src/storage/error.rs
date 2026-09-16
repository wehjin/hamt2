use crate::trie::SlotBaseId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadStorageError {
    #[error("failed to read base id {0} from disk: {1}")]
    Io(SlotBaseId, #[source] std::io::Error),
    #[error("failed to decode base id {0}: {1}")]
    Decode(SlotBaseId, #[source] postcard::Error),
}

#[derive(Debug, Error)]
pub enum WriteStorageError {
    #[error("failed to write base id {0} to disk: {1}")]
    Io(SlotBaseId, #[source] std::io::Error),
    #[error("failed to encode base id {0}: {1}")]
    Encode(SlotBaseId, #[source] postcard::Error),
}
