use crate::QueryCursor;
use crate::{Query, Snap};

pub trait Trie:
    QueryCursor
    + Snap<Snapshot = Self>
    + Query
    + Send
    + Sync
    + Eq
    + PartialEq
    + Clone
{
}
