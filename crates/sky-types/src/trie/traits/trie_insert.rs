use crate::trie::map_base::query_value;
use crate::trie::{BaseCommit, DeepKey, HashKey, MapBase, TrieInsertError, TrieValue, map_base};
use std::collections::HashMap;

impl<T: BaseCommit> TrieInsert for T {
    async fn insert(&mut self, key: i32, value: TrieValue) -> Result<&mut Self, TrieInsertError> {
        let key = HashKey::new(key);
        let root = map_base::insert_kv(self.read_root(), key, value, self).await?;
        self.commit_root(root).await?;
        Ok(self)
    }

    async fn deep_insert<const N: usize>(
        &mut self,
        key: [i32; N],
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<&mut Self, TrieInsertError> {
        let deep_key = DeepKey::from(key);
        let last_index = N - 1;
        let mut map_bases = HashMap::new();
        map_bases.insert(0, self.read_root().clone());
        for i in 0..last_index {
            let key = deep_key[i].clone();
            let map_base = map_bases.get(&i).expect("map_base should exist");
            let subtrie_i = i + 1;
            let map_base_i = if replace_tail && subtrie_i == last_index {
                MapBase::empty()
            } else {
                match query_value(*map_base, key, self).await? {
                    None => MapBase::empty(),
                    Some(TrieValue::SubTrie(map_base)) => map_base,
                    Some(TrieValue::U32(_)) => unreachable!("expected a sub-trie but found a u32"),
                }
            };
            map_bases.insert(subtrie_i, map_base_i);
        }
        let mut value = value.into();
        for i in (0..=last_index).rev() {
            let key = deep_key[i].clone();
            let pre_map_base = map_bases.get(&i).expect("map_base should exist");
            let post_map_base = map_base::insert_kv(pre_map_base.clone(), key, value, self).await?;
            value = TrieValue::SubTrie(post_map_base);
        }
        let TrieValue::SubTrie(root) = value else {
            panic!("value should be map_base")
        };
        self.commit_root(root).await?;
        Ok(self)
    }
}

#[allow(async_fn_in_trait)]
pub trait TrieInsert: BaseCommit {
    async fn insert(&mut self, key: i32, value: TrieValue) -> Result<&mut Self, TrieInsertError>;

    async fn deep_insert<const N: usize>(
        &mut self,
        key: [i32; N],
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<&mut Self, TrieInsertError>;
}
