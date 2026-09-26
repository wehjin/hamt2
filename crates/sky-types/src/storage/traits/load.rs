use crate::trie::TrieWalk;
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

pub trait StoreLoad:
    TrieWalk<Subtrie = Self> + TrieStream + TrieSnap<Snapshot = Self> + TrieQuery + Send
{
}