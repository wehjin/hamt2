use crate::hamt::trie::core::deep_key::DeepKey;
use crate::hamt::trie::core::key::TrieKey;
use crate::hamt::trie::core::map_base::TrieMapBase;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::space::SpaceRoot;
use crate::space::{Read, Space};
use crate::{QueryError, TransactError};
use std::collections::HashMap;

pub mod query;

#[derive(Debug)]
pub struct SpaceTrie<T: Space> {
    map_base: TrieMapBase,
    reader: T::Reader,
}

impl<T: Space> SpaceTrie<T> {
    pub fn connect(space: &T) -> Result<Self, QueryError> {
        let reader = space.read()?;
        let map_base = match reader.read_root()? {
            None => TrieMapBase::empty(),
            Some(root) => TrieMapBase::from(reader.read_slot(&root, 0)?),
        };
        let trie = Self { map_base, reader };
        Ok(trie)
    }

    pub fn commit(self, space: &mut T) -> Result<(), TransactError> {
        let mut extend = space.extend()?;
        let root_addr = {
            let space_map_base = self.map_base.into_space_map_base(&mut extend)?;
            let (map, base_addr) = space_map_base.into_map_base_addr();
            let space_root = SpaceRoot(map, base_addr);
            let root_addr = space_root.into_root_addr(&mut extend)?;
            root_addr
        };
        extend.set_root(root_addr);
        extend.commit(space)
    }

    pub fn to_subtrie_from_value(&self, value: MemValue) -> Result<Self, QueryError> {
        let map_base = match value {
            MemValue::MapBase(map_base) => map_base,
            MemValue::U32(u32) => {
                SpaceRoot::from_root_addr(u32, &self.reader)?.into_trie_map_base()
            }
        };
        let subtrie = self.to_subtrie(map_base);
        Ok(subtrie)
    }

    pub fn to_subtrie(&self, map_base: TrieMapBase) -> Self {
        Self {
            map_base,
            reader: self.reader.clone(),
        }
    }

    pub fn insert(self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let map_base = self.map_base.insert_kv(key, value, &self.reader)?;
        Ok(Self {
            map_base,
            reader: self.reader,
        })
    }

    pub fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        self.map_base.query_value(key, &self.reader)
    }

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
                    let map_base = match value {
                        MemValue::MapBase(map_base) => map_base,
                        MemValue::U32(u32) => {
                            SpaceRoot::from_root_addr(u32, &self.reader)?.into_trie_map_base()
                        }
                    };
                    map_bases.insert(i + 1, map_base);
                }
            }
        }
        let mut value = value;
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let post_map_base = map_base.clone().insert_kv(key, value, &self.reader)?;
            value = MemValue::MapBase(post_map_base);
        }
        let MemValue::MapBase(map_base) = value else {
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
                    if i == last_index {
                        return Ok(Some(value));
                    } else {
                        let map_base = match value {
                            MemValue::MapBase(map_base) => map_base,
                            MemValue::U32(u32) => {
                                SpaceRoot::from_root_addr(u32, &self.reader)?.into_trie_map_base()
                            }
                        };
                        current_map_base = map_base;
                    }
                }
            }
        }
        unreachable!();
    }
}
