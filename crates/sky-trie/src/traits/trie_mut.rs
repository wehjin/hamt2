use crate::{Insert, InsertCursor, KvStream, Query, QueryCursor, Snap};

/// Deliberately no clone.
pub trait TrieMut:
    InsertCursor + Insert + QueryCursor + KvStream + Snap<Snapshot = Self> + Query + Send
{
}
