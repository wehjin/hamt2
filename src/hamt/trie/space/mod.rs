use crate::client::{QueryError, TransactError};
use crate::hamt::trie::deep_key::DeepKey;
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map_base::TrieMapBase;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::value::TrieValue;
use std::collections::HashMap;

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

    pub fn deep_insert<const N: usize>(
        &mut self,
        key: [i32; N],
        value: MemValue,
    ) -> Result<(), TransactError> {
        let deep_key = DeepKey::from(key);
        let last_index = N - 1;
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.map_base.clone());
        for i in 0..last_index {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            match map_base.query_value(key)? {
                None => {
                    map_bases.insert(i + 1, TrieMapBase::empty());
                }
                Some(TrieValue::Mem(value)) => match value {
                    MemValue::U32(_) => {
                        return Err(TransactError::ExpectedMapBaseAtKey);
                    }
                    MemValue::MapBase(map_base) => {
                        map_bases.insert(i + 1, map_base);
                    }
                },
            }
        }
        let mut value = TrieValue::Mem(value);
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let map_base = map_bases
                .get(&i)
                .expect("map_base should exist")
                .clone()
                .insert_kv(key, value)?;
            value = TrieValue::Mem(MemValue::MapBase(map_base));
        }
        let TrieValue::Mem(MemValue::MapBase(map_base)) = value else {
            panic!("value should be map_base")
        };
        self.map_base = map_base;
        Ok(())
    }

    pub fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<MemValue>, QueryError> {
        let deep_key = DeepKey::from(key);
        let mut current_map_base = self.map_base.clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match current_map_base.query_value(deep_key[i].clone())? {
                Some(TrieValue::Mem(value)) => {
                    if i == last_index {
                        return Ok(Some(value));
                    } else {
                        let MemValue::MapBase(map_base) = value else {
                            return Err(QueryError::NoSubtrieAtKeyIndex(i));
                        };
                        current_map_base = map_base;
                    }
                }
                None => {
                    return Ok(None);
                }
            }
        }
        unreachable!();
    }
}
