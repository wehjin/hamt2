use sky_types::db::Attr;
use sky_types::storage::error::ReadStorageError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Trie storage read: {0}")]
    TrieStorageRead(#[from] ReadStorageError),

    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
