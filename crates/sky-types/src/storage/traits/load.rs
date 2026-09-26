use crate::trie::TrieWalk;
use crate::trie::{TrieQuery, TrieSnap, TrieStream};

pub trait StoreLoad: TrieWalk + TrieStream + TrieSnap + TrieQuery + Send {}
