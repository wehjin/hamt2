use crate::{KvStream, Query, QueryCursor};

pub trait Snap: Sized {
    type Snapshot: QueryCursor + KvStream + Snap + Query + Send;

    /// Returns an owned snapshot of this read-only storage that is also a read-only
    /// storage. Future writes to the original MUST NOT affect the snapshot.
    fn snapshot(&self) -> Self::Snapshot;
}
