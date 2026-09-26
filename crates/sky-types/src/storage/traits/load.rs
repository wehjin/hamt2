use crate::trie::{TrieQuery, TrieSnap, TrieStream, TrieWalk};

pub trait StoreLoad:
    TrieWalk<Subtrie = Self> + TrieStream + TrieSnap<Snapshot = Self> + TrieQuery + Send
{
}