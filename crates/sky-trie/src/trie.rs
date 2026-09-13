use crate::TrieQuery;
use crate::trie_ref::TrieRef;
use crate::trie_storage::ReadWriteTrieStorage;
use crate::trie_storage::errors::{TrieStorageReadError, TrieStorageWriteError};
use crate::types::map_base::MapBase;
use std::collections::HashMap;

use crate::TrieWriteError;
use crate::prelude::TrieValue;
use crate::types::hash_key::HashKey;
use crate::types::hash_key_path::HashKeyPath;

#[derive(Debug)]
pub struct Trie<S: ReadWriteTrieStorage> {
    root: MapBase,
    storage: S,
}

impl<S: ReadWriteTrieStorage> TrieQuery<S> for Trie<S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        &self.storage
    }
}

/// Trie construction methods.
impl<S: ReadWriteTrieStorage> Trie<S> {
    /// Connects to the storage, loading the persisted root if there is one.
    pub async fn connect(storage: S) -> Result<Self, TrieStorageReadError> {
        let root = storage.get_root().await?;
        Ok(Self { root, storage })
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(mut self) -> Result<Self, TrieStorageWriteError> {
        self.storage.write_root(self.root.clone()).await?;
        Ok(self)
    }

    pub fn unwrap(self) -> MapBase {
        self.root
    }

    pub fn close(self) -> S {
        self.storage
    }

    /// A borrowed read-only view of this trie.
    pub fn view(&self) -> TrieRef<'_, S> {
        TrieRef::new(self.root.clone(), &self.storage)
    }
}

/// Trie update methods.
impl<S: ReadWriteTrieStorage> Trie<S> {
    pub async fn insert(mut self, key: i32, value: TrieValue) -> Result<Self, TrieWriteError> {
        let key = HashKey::new(key);
        let root = self.root.insert_kv(key, value, &mut self.storage).await?;
        self.root = root;
        Ok(self)
    }

    pub async fn deep_insert<const N: usize>(
        mut self,
        key: [i32; N],
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<Self, TrieWriteError> {
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
                        return Err(TrieWriteError::ExpectedMapBaseAtKey);
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
