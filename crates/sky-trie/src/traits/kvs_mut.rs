use crate::{Insert, InsertCursor, KvStream, Query, QueryCursor, Snap};

/// Deliberately no clone.
pub trait KvsMut:
    InsertCursor + Insert + QueryCursor + KvStream + Snap<Snapshot = Self> + Query + Send
{
}
