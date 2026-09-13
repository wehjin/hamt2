use crate::TransactError;
use crate::trie::Trie;
use crate::trie::trie_storage::ReadWriteTrieStorage;
use crate::trie::types::hash_key_path::HashKeyPath;
use crate::trie::types::map_base::MapBase;
use crate::trie::types::trie_value::TrieValue;
use std::collections::HashMap;

impl<S: ReadWriteTrieStorage> Trie<S> {
    pub async fn deep_insert<const N: usize>(
	    mut self,
	    key: [i32; N],
	    value: impl Into<TrieValue>,
	    replace_tail: bool,
    ) -> Result<Self, TransactError> {
        let deep_key = HashKeyPath::from(key);
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
                    Some(TrieValue::SubTrie(map_base)) => map_base,
                    Some(TrieValue::U32(_)) => {
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
            value = TrieValue::SubTrie(post_map_base);
        }
        let TrieValue::SubTrie(root) = value else {
            panic!("value should be map_base")
        };
        self.root = root;
        Ok(self)
    }
}
