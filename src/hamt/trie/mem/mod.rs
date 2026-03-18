use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map_base::TrieMapBase;
use crate::hamt::trie::value::TrieValue;

pub mod base;
pub mod slot;
pub mod value;

#[derive(Debug)]
pub struct MemTrie {
    pub root_map_base: Option<TrieMapBase>,
}

impl MemTrie {
    pub fn empty() -> Self {
        Self {
            root_map_base: None,
        }
    }
    pub fn one_kv(key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        Self::empty().insert(key, value)
    }
    pub fn insert(self, key: TrieKey, value: TrieValue) -> Result<Self, TransactError> {
        let root_map_base = if let Some(map_base) = self.root_map_base {
            map_base.insert_kv(key, value)?
        } else {
            TrieMapBase::one_kv(key, value)?
        };
        Ok(Self {
            root_map_base: Some(root_map_base),
        })
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<TrieValue>, QueryError> {
        if let Some(map_base) = &self.root_map_base {
            map_base.query_value(key)
        } else {
            Ok(None)
        }
    }
}
