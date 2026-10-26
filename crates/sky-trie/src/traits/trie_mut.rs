use crate::{Insert, InsertCursor, Query, QueryCursor, Snap};

/// Deliberately no clone.
pub trait TrieMut:
    InsertCursor + Insert + QueryCursor + Snap + Query + Send
{
}
