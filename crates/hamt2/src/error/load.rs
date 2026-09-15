use crate::db::Attr;
use crate::trie::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadError {
    #[error("Trie storage read: {0}")]
    TrieStorageRead(#[from] StorageReadError),

    #[error("Unknown attribute: {0:?}")]
    UnknownAttr(Attr),
}
