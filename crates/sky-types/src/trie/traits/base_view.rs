use crate::trie::{BaseRead, MapBase};

/// These are the core functions of a read-only trie.
#[allow(async_fn_in_trait)]
pub trait BaseView: BaseRead + Sized {
    type Snapshot: BaseView + Send + Clone;

    /// Convert the storage into one that reads starting at the new root. The new
    /// root should exist within the existing root's tree.
    fn with_new_root(self, new_root: Option<MapBase>) -> Self;

    /// Returns an owned snapshot of this read-only storage that is also a read-only
    /// storage. Future writes to the original MUST NOT affect the snapshot.
    fn snapshot(&self) -> Self::Snapshot;
}
