use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::mem::MemTrie;

pub mod core;

pub struct SpaceTrie {
    mem_trie: Option<MemTrie>,
}

impl SpaceTrie {
    pub fn new() -> Self {
        Self { mem_trie: None }
    }

    pub fn insert(self, key: TrieKey, value: u32) -> Result<Self, TransactError> {
        if let Some(_root) = &self.mem_trie {
            unimplemented!()
        } else {
            let mem_trie = MemTrie::one_kv(key, value)?;
            Ok(Self {
                mem_trie: Some(mem_trie),
            })
        }
    }

    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        if let Some(mem_trie) = &self.mem_trie {
            let value = mem_trie.query_value(key)?;
            Ok(value)
        } else {
            Ok(None)
        }
    }
}
