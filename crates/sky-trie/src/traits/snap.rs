use crate::{QueryCursor, Query, KvStream};

/// These are the core functions of a read-only trie.
#[allow(async_fn_in_trait)]
pub trait Snap: Sized {
    type Snapshot: QueryCursor + KvStream + Snap + Query + Send;

    /// Returns an owned snapshot of this read-only storage that is also a read-only
    /// storage. Future writes to the original MUST NOT affect the snapshot.
    fn snapshot(&self) -> Self::Snapshot;
}
