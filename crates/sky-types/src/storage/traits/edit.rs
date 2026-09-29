use crate::trie::{
    InsertCursor, QueryCursor, TrieInsert, TrieQuery, TrieSnap, TrieStream, TrieWalk,
};

/// Deliberately no clone.
pub trait StoreEdit:
    InsertCursor
    + TrieInsert
    + TrieWalk<Subtrie = Self>
    + QueryCursor
    + TrieStream
    + TrieSnap<Snapshot = Self>
    + TrieQuery
    + Send
{
}
