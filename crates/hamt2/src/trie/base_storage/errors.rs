use crate::trie::base::BaseId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaseStorageReadError {
    #[error("base id {0} has not been written")]
    NotFound(BaseId),
    #[error("failed to read base id {0} from disk: {1}")]
    Io(BaseId, #[source] std::io::Error),
    #[error("failed to decode base id {0}: {1}")]
    Decode(BaseId, #[source] postcard::Error),
}

#[derive(Debug, Error)]
pub enum BaseStorageWriteError {
    #[error("failed to write base id {0} to disk: {1}")]
    Io(BaseId, #[source] std::io::Error),
    #[error("failed to encode base id {0}: {1}")]
    Encode(BaseId, #[source] postcard::Error),
}