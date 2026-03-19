use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map_base::TrieMapBase;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::value::TrieValue;

pub mod core;

#[derive(Debug)]
pub struct SpaceTrie {
    map_base: TrieMapBase,
}

impl SpaceTrie {
    pub fn new() -> Self {
        Self {
            map_base: TrieMapBase::empty(),
        }
    }

    pub fn insert(self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let value = TrieValue::Mem(value);
        let map_base = self.map_base.insert_kv(key, value)?;
        Ok(Self { map_base })
    }

    pub fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        let value = self
            .map_base
            .query_value(key)?
            .map(|TrieValue::Mem(value)| value);
        Ok(value)
    }
}
