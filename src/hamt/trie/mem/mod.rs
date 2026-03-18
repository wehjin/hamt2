use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::value::TrieValue;
use map_base::MemMapBase;

pub mod base;
pub mod map_base;
pub mod slot;
pub mod value;

#[derive(Debug)]
pub struct MemTrie {
    pub root_map_base: Option<MemMapBase>,
}

impl MemTrie {
    pub fn empty() -> Self {
        Self {
            root_map_base: None,
        }
    }
    pub fn one_kv(key: i32, value: u32) -> Result<Self, TransactError> {
        Self::empty().insert(key, value)
    }
    pub fn insert(self, key: i32, value: u32) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let value = TrieValue::new(value)?;
        let root_map_base = if let Some(map_base) = self.root_map_base {
            map_base.insert_kv(key, value)?
        } else {
            MemMapBase::one_kv(key, value)?
        };
        Ok(Self {
            root_map_base: Some(root_map_base),
        })
    }
    pub fn query_value(&self, key: i32) -> Result<Option<u32>, QueryError> {
        let key = TrieKey::new(key);
        if let Some(root_map_base) = &self.root_map_base {
            root_map_base.query_value(key)
        } else {
            Ok(None)
        }
    }
}
