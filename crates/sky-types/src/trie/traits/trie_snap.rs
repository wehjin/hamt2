use crate::trie::{TrieQuery, TrieStream, TrieWalk};

/// These are the core functions of a read-only trie.
#[allow(async_fn_in_trait)]
pub trait TrieSnap: Sized {
    type Snapshot: TrieWalk + TrieStream + TrieSnap + TrieQuery + Send + Clone;

    /// Returns an owned snapshot of this read-only storage that is also a read-only
    /// storage. Future writes to the original MUST NOT affect the snapshot.
    fn snapshot(&self) -> Self::Snapshot;
}
