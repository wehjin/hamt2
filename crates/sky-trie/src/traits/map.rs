use crate::QueryCursor;
use crate::{Query, Snap, KvStream};

pub trait Map:
    QueryCursor
    + KvStream
    + Snap<Snapshot = Self>
    + Query
    + Send
    + Sync
    + Eq
    + PartialEq
    + Clone
{
}
