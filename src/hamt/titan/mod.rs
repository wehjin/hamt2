use crate::client::{QueryError, TransactError};
use crate::hamt::trie::key::TrieKey;
use crate::hamt::trie::map_base::TrieMapBase;
use crate::hamt::trie::mem::value::MemValue;
use crate::hamt::trie::value::TrieValue;
use std::collections::HashMap;

pub struct Titan {
    map_base: TrieMapBase,
}
impl Titan {
    pub fn new() -> Self {
        Self {
            map_base: TrieMapBase::empty(),
        }
    }
    pub fn insert<const N: usize>(
        &mut self,
        key: [TrieKey; N],
        value: MemValue,
    ) -> Result<(), TransactError> {
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.map_base.clone());
        let last_index = N - 1;
        for i in 0..last_index {
            let key = key[i].clone();
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
            let key = key[i].clone();
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

    pub fn query_value<const N: usize>(
        &self,
        key: [TrieKey; N],
    ) -> Result<Option<MemValue>, QueryError> {
        let mut current_map_base = self.map_base.clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match current_map_base.query_value(key[i].clone())? {
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

#[cfg(test)]
mod tests {
    use crate::client::QueryError;
    use crate::hamt::titan::Titan;
    use crate::hamt::trie::key::TrieKey;
    use crate::hamt::trie::mem::value::MemValue;

    #[tokio::test]
    async fn titan_works() {
        let mut trie = Titan::new();
        trie.insert([TrieKey::new(4), TrieKey::new(2)], MemValue::U32(42))
            .unwrap();
        {
            let value = trie.query_value([TrieKey::new(4)]).unwrap();
            let Some(MemValue::MapBase(map_base)) = value else {
                panic!("expected map_base");
            };
            assert_eq!(1, map_base.map.len());
            assert_eq!(1, map_base.base.len());
        }
        {
            let value = trie
                .query_value([TrieKey::new(4), TrieKey::new(2)])
                .unwrap();
            assert_eq!(Some(MemValue::U32(42)), value);
        }
        {
            let value = trie
                .query_value([TrieKey::new(4), TrieKey::new(1)])
                .unwrap();
            assert_eq!(None, value);
        }
        {
            let value = trie
                .query_value([TrieKey::new(5), TrieKey::new(2)])
                .unwrap();
            assert_eq!(None, value);
        }
        {
            let result = trie.query_value([TrieKey::new(4), TrieKey::new(2), TrieKey::new(-1)]);
            let Err(QueryError::NoSubtrieAtKeyIndex(1)) = result else {
                panic!("expected NoSubtrieAtKeyIndex(1)")
            };
        }
    }
}
