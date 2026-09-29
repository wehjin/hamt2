use crate::trie::{QueryCursor, TrieWalk};
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

pub trait StoreView:
    TrieWalk<Subtrie = Self>
    + QueryCursor
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
