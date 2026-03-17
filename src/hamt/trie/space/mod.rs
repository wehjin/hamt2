use crate::client::{QueryError, TransactError};
use crate::hamt::trie::mem::MemTrie;

pub mod core;

pub struct SpaceTrie {
    mem_trie: Option<MemTrie>,
}

impl SpaceTrie {
    pub fn new() -> Self {
        Self { mem_trie: None }
    }

    pub fn insert(self, key: i32, value: u32) -> Result<Self, TransactError> {
        let mem_trie = if let Some(mem_trie) = self.mem_trie {
            mem_trie.insert(key, value)?
        } else {
            MemTrie::one_kv(key, value)?
        };
        Ok(Self {
            mem_trie: Some(mem_trie),
        })
    }

    pub fn query_value(&self, key: i32) -> Result<Option<u32>, QueryError> {
        if let Some(mem_trie) = &self.mem_trie {
            let value = mem_trie.query_value(key)?;
            Ok(value)
        } else {
            Ok(None)
        }
    }
}
