use crate::trie::QueryCursor;
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

pub trait StoreView:
    QueryCursor
    + TrieStream
    + TrieSnap<Snapshot = Self>
    + TrieQuery
    + Send
    + Sync
    + Eq
    + PartialEq
    + Clone
{
}
