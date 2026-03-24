use crate::hamt::trie::core::deep_key::DeepKey;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::core::value::TrieValue;
use crate::hamt::trie::mem::value::MemValue;
use crate::space;
use crate::space::table::TableRoot;
use crate::space::Read;
use crate::space::Space;
use crate::QueryError;
use crate::TransactError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SpaceTrie {
    map_base: TrieMapBase,
    reader: space::Reader,
}

impl SpaceTrie {
    pub fn connect<T: Space>(space: &T) -> Result<Self, QueryError> {
        let reader = space.read()?;
        let map_base = match reader.read_root()? {
            None => TrieMapBase::empty(),
            Some(root) => {
                let map_base = TrieMapBase::from(root);
                map_base
            }
        };
        let trie = Self { map_base, reader };
        Ok(trie)
    }

    pub fn commit<T: Space>(self, space: &mut T) -> Result<(), TransactError> {
        let mut extend = space.extend();
        let (map, base_addr) = self.map_base.into_map_base_addr(&mut extend)?;
        extend.set_root(TableRoot(map.0, base_addr));
        extend.commit(space)
    }

    pub fn to_subtrie(&self, map_base: TrieMapBase) -> Self {
        Self {
            map_base,
            reader: self.reader.clone(),
        }
    }

    pub fn query_key_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.map_base.query_key_values(&self.reader)
    }
}

impl SpaceTrie {
    pub fn insert(self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let value = TrieValue::Mem(value);
        let map_base = self.map_base.insert_kv(key, value, &self.reader)?;
        Ok(Self {
            map_base,
            reader: self.reader,
        })
    }

    pub fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        match self.map_base.query_value(key, &self.reader)? {
            None => Ok(None),
            Some(value) => {
                let value = value.to_mem_value(&self.reader)?;
                Ok(Some(value))
            }
        }
    }
}

impl SpaceTrie {
    pub fn deep_insert<const N: usize>(
        self,
        key: [i32; N],
        value: MemValue,
    ) -> Result<Self, TransactError> {
        let deep_key = DeepKey::from(key);
        let last_index = N - 1;
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.map_base.clone());
        for i in 0..last_index {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            match map_base.query_value(key, &self.reader)? {
                None => {
                    map_bases.insert(i + 1, TrieMapBase::empty());
                }
                Some(value) => {
                    let mem_value = value
                        .to_mem_value(&self.reader)
                        .map_err(|e| TransactError::SpaceReadError(e))?;
                    let MemValue::MapBase(map_base) = mem_value else {
                        return Err(TransactError::ExpectedMapBaseAtKey);
                    };
                    map_bases.insert(i + 1, map_base);
                }
            }
        }
        let mut value = TrieValue::Mem(value);
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let map_base = map_bases
                .get(&i)
                .expect("map_base should exist")
                .clone()
                .insert_kv(key, value, &self.reader)?;
            value = TrieValue::Mem(MemValue::MapBase(map_base));
        }
        let TrieValue::Mem(MemValue::MapBase(map_base)) = value else {
            panic!("value should be map_base")
        };
        Ok(Self {
            map_base,
            reader: self.reader,
        })
    }

    pub fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<MemValue>, QueryError> {
        let deep_key = DeepKey::from(key);
        let mut current_map_base = self.map_base.clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match current_map_base.query_value(deep_key[i].clone(), &self.reader)? {
                None => {
                    return Ok(None);
                }
                Some(value) => {
                    let value = value.to_mem_value(&self.reader)?;
                    if i == last_index {
                        return Ok(Some(value));
                    } else {
                        let MemValue::MapBase(map_base) = value else {
                            return Err(QueryError::NoSubtrieAtKeyIndex(i));
                        };
                        current_map_base = map_base;
                    }
                }
            }
        }
        unreachable!();
    }
}
