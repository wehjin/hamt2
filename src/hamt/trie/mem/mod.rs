use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use core::MemMapBase;

pub mod core;

pub struct MemTrie {
    pub root_map_base: Option<MemMapBase>,
}

impl MemTrie {
    pub fn empty() -> Self {
        Self {
            root_map_base: None,
        }
    }
    pub fn one_kv(key: TrieKey, value: u32) -> Result<Self, TransactError> {
        Self::empty().insert(key, value)
    }
    pub fn insert(self, key: TrieKey, value: u32) -> Result<Self, TransactError> {
        if let Some(_root) = self.root_map_base {
            unimplemented!()
        } else {
            let root = MemMapBase::one_kv(key, value)?;
            Ok(Self {
                root_map_base: Some(root),
            })
        }
    }
    pub fn query_value(&self, key: TrieKey) -> Result<Option<u32>, QueryError> {
        if let Some(root_map_base) = &self.root_map_base {
            root_map_base.query_value(key)
        } else {
            Ok(None)
        }
    }
}
