use crate::trie::{TrieQuery, TrieSnap, TrieStream, TrieWalk};

pub trait StoreView:
    TrieWalk<Subtrie = Self> + TrieStream + TrieSnap<Snapshot = Self> + TrieQuery + Send + Sync + Clone
{
}