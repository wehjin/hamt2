use crate::storage::ReadStorage;
use crate::trie::{BaseId, TrieCommit};

/// A trait for reading and writing Bases from storage.
#[allow(async_fn_in_trait)]
pub trait ReadWriteStorage: TrieCommit + ReadStorage {
    /// Read the next available base id. The value is 1 in an empty storage because base id 0 is reserved for the empty base.
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }
}
