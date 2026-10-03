use crate::QueryCursor;
use crate::{Query, Snap, KvStream};

pub trait Trie:
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
