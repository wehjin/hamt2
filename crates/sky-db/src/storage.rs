//! The db storage surface re-exported from [`sky_trie`], renamed so callers
//! never need to name trie types.
//!
//! sky-db imports the rest of `sky_trie` privately (as `crate::trie`); these
//! are the only storage types that public sky-db APIs require callers to name.

pub use sky_types::storage::error::{
	StorageReadError as DbStorageReadError, StorageWriteError as DbStorageWriteError,
};
pub use sky_trie::storage::file::FileTrieStorage as FileDbStorage;
pub use sky_trie::storage::mem::MemTrieStorage as MemDbStorage;
pub use sky_trie::storage::{
    ReadTrieStorage as ReadDbStorage, ReadWriteTrieStorage as ReadWriteDbStorage,
};
