use std::collections::HashMap;
use crate::space::Space;
use crate::{QueryError, TransactError};
use crate::trie::core::deep_key::DeepKey;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::space::root::SpaceRoot;
use crate::trie::SpaceTrie;

impl<T: Space> SpaceTrie<T> {
    pub async fn deep_insert<const N: usize>(
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
            match map_base.query_value(key, &self.reader).await? {
                None => {
                    map_bases.insert(i + 1, TrieMapBase::empty());
                }
                Some(value) => {
                    let map_base = match value {
                        MemValue::MapBase(map_base) => map_base,
                        MemValue::U32(u32) => SpaceRoot::from_root_addr(u32, &self.reader)
                            .await?
                            .into_trie_map_base(),
                    };
                    map_bases.insert(i + 1, map_base);
                }
            }
        }
        let mut value = value;
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let post_map_base = map_base.clone().insert_kv(key, value, &self.reader).await?;
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

    pub async fn deep_query_value<const N: usize>(
        &self,
        key: [i32; N],
    ) -> Result<Option<MemValue>, QueryError> {
        let deep_key = DeepKey::from(key);
        let mut current_map_base = self.map_base.clone();
        let last_index = N - 1;
        for i in 0..=last_index {
            match current_map_base
                .query_value(deep_key[i].clone(), &self.reader)
                .await?
            {
                None => {
                    return Ok(None);
                }
                Some(value) => {
                    if i == last_index {
                        return Ok(Some(value));
                    } else {
                        let map_base = match value {
                            MemValue::MapBase(map_base) => map_base,
                            MemValue::U32(u32) => SpaceRoot::from_root_addr(u32, &self.reader)
                                .await?
                                .into_trie_map_base(),
                        };
                        current_map_base = map_base;
                    }
                }
            }
        }
        unreachable!();
    }
}