use crate::trie::Trie;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::deep_key::DeepKey;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::value::MemValue;
use crate::TransactError;
use std::collections::HashMap;

impl<S: BaseStorageReadWrite> Trie<S> {
    pub async fn deep_insert<const N: usize>(
        mut self,
        key: [i32; N],
        value: impl Into<MemValue>,
        replace_tail: bool,
    ) -> Result<Self, TransactError> {
        let deep_key = DeepKey::from(key);
        let last_index = N - 1;
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.root.clone());
        for i in 0..last_index {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let subtrie_i = i + 1;
            let map_base_i = if replace_tail && subtrie_i == last_index {
                MapBase::empty()
            } else {
                match map_base.query_value(key, &self.storage).await? {
                    None => MapBase::empty(),
                    Some(MemValue::MapBase(map_base)) => map_base,
                    Some(MemValue::U32(_)) => {
                        return Err(TransactError::ExpectedMapBaseAtKey);
                    }
                }
            };
            map_bases.insert(subtrie_i, map_base_i);
        }
        let mut value = value.into();
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let post_map_base = map_base
                .clone()
                .insert_kv(key, value, &mut self.storage)
                .await?;
            value = MemValue::MapBase(post_map_base);
        }
        let MemValue::MapBase(root) = value else {
            panic!("value should be map_base")
        };
        self.root = root;
        Ok(self)
    }
}
