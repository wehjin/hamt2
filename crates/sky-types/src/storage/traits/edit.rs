use crate::trie::{TrieInsert, TrieQuery, TrieSnap, TrieStream, TrieWalk};

/// Deliberately no clone.
pub trait StoreEdit:
    TrieInsert + TrieWalk<Subtrie = Self> + TrieStream + TrieSnap<Snapshot = Self> + TrieQuery + Send
{
}
