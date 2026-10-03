use crate::{Insert, InsertCursor, KvStream, Query, QueryCursor, Snap};

/// Deliberately no clone.
pub trait MapMut:
    InsertCursor + Insert + QueryCursor + KvStream + Snap<Snapshot = Self> + Query + Send
{
}
