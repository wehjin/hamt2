use crate::client::{QueryError, TransactError};
use crate::hamt::trie::core::TrieValue;
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::mem::core::KvTest;
use core::MemMapBase;

pub mod core;

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
        let root_map_base = if let Some(root_map_base) = self.root_map_base {
            match root_map_base.as_slot(key)? {
                Some(slot) => match slot.test_kv(key, value) {
                    KvTest::SameValue => root_map_base,
                    KvTest::ConflictOldValue(_) => root_map_base.replace_existing_value(key, value)?,
                    KvTest::ConflictKeyValue(_, _) => root_map_base.merge_kv(key, value)?,
                    KvTest::ConflictMapBase => root_map_base.merge_kv(key, value)?,
                },
                None => root_map_base.insert_kv(key, value)?,
            }
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
