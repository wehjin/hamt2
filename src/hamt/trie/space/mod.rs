use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::mem::MemTrie;
use crate::hamt::trie::value::TrieValue;

pub mod core;

#[derive(Debug)]
pub struct SpaceTrie {
    mem_trie: Option<MemTrie>,
}

impl SpaceTrie {
    pub fn new() -> Self {
        Self { mem_trie: None }
    }

    pub fn insert(self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let value = TrieValue::Mem(value);
        let mem_trie = if let Some(mem_trie) = self.mem_trie {
            mem_trie.insert(key, value)?
        } else {
            MemTrie::one_kv(key, value)?
        };
        Ok(Self {
            mem_trie: Some(mem_trie),
        })
    }

    pub fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        if let Some(mem_trie) = &self.mem_trie {
            let value = mem_trie.query_value(key)?.map(|TrieValue::Mem(v)| v);
            Ok(value)
        } else {
            Ok(None)
        }
    }
}
