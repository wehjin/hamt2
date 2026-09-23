use crate::storage::TrieView;
use crate::trie::{BaseCommit, BaseId};

/// A trait for reading and writing Bases from storage.
#[allow(async_fn_in_trait)]
pub trait TrieEdit: BaseCommit + TrieView {
    /// Read the next available base id.
    fn next_id(&self) -> BaseId {
        self.max_id() + 1
    }
}
