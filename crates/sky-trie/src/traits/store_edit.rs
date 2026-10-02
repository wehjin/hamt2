use crate::{InsertCursor, QueryCursor, TrieInsert, TrieQuery, TrieSnap, TrieStream};

/// Deliberately no clone.
pub trait StoreEdit:
    InsertCursor + TrieInsert + QueryCursor + TrieStream + TrieSnap<Snapshot = Self> + TrieQuery + Send
{
}
