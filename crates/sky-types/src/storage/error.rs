use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadStorageError {
    #[error("failed to read {0} from disk: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("failed to decode {0}: {1}")]
    Decode(String, #[source] postcard::Error),
}

#[derive(Debug, Error)]
pub enum WriteStorageError {
    #[error("failed to write {0} to disk: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("failed to encode {0}: {1}")]
    Encode(String, #[source] postcard::Error),
}
