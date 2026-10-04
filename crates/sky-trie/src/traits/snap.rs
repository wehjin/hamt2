use crate::{Query, QueryCursor};

pub trait Snap: Sized {
    type Snapshot: QueryCursor + Snap + Query + Send;

    /// Returns an owned snapshot of this read-only trie that is also a read-only
    /// trie. Future writes to the original MUST NOT affect the snapshot.
    fn snapshot(&self) -> Self::Snapshot;
}
