use crate::trie::{TrieInsert, TrieQuery};

/// Deliberately no clone.
pub trait StoreEdit: TrieInsert + TrieQuery + Send {}