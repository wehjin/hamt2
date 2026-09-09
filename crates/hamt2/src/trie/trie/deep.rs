use crate::space::Space;
use crate::trie::core::deep_key::DeepKey;
use crate::trie::mem::value::MemValue;
use crate::trie::space::root::SpaceRoot;
use crate::trie::SpaceTrie;
use crate::{QueryError, TransactError};
use std::collections::HashMap;

impl<T: Space> SpaceTrie<T> {
    pub async fn deep_insert<const N: usize>(
        self,
        key: [i32; N],
        value: impl Into<MemValue>,
        replace_tail: bool,
    ) -> Result<Self, TransactError> {
        let deep_key = DeepKey::from(key);
        let last_index = N - 1;
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.map_base.clone());
        for i in 0..last_index {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let subtrie_i = i + 1;
            let subtrie = if replace_tail && subtrie_i == last_index {
                self.new_subtrie()
            } else {
                match map_base.query_value(key, &self.reader).await? {
                    None => self.new_subtrie(),
                    Some(value) => self.to_subtrie_from_value(value).await?,
                }
            };
            map_bases.insert(subtrie_i, subtrie.unwrap());
        }
        let mut value = value.into();
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
                    if i < last_index {
                        let map_base = match value {
                            MemValue::MapBase(map_base) => map_base,
                            MemValue::U32(u32) => SpaceRoot::from_root_addr(u32, &self.reader)
                                .await?
                                .into_trie_map_base(),
                        };
                        current_map_base = map_base;
                    } else {
                        return Ok(Some(value));
                    }
                }
            }
        }
        unreachable!();
    }
}
