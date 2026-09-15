//! The db storage surface re-exported from [`sky_trie`], renamed so callers
//! never need to name trie types.
//!
//! hamt2 imports the rest of `sky_trie` privately (as `crate::trie`); these
//! are the only storage types that public hamt2 APIs require callers to name.

pub use sky_trie::trie_storage::errors::{
	StorageReadError as DbStorageReadError, StorageWriteError as DbStorageWriteError,
};
pub use sky_trie::trie_storage::file::FileTrieStorage as FileDbStorage;
pub use sky_trie::trie_storage::mem::MemTrieStorage as MemDbStorage;
pub use sky_trie::trie_storage::{
    ReadTrieStorage as ReadDbStorage, ReadWriteTrieStorage as ReadWriteDbStorage,
};
