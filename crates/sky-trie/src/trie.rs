use crate::TrieReader;
use crate::prelude::TrieValue;
use sky_types::storage::error::WriteStorageError;
use sky_types::storage::{ReadWriteStorage, Storage, StorageHead, StoreRead};
use sky_types::trie::map_base::query_value;
use sky_types::trie::{
    DeepKey, HashKey, MapBase, SlotBase, SlotBaseId, TrieBaseRead, TrieQueryError,
};
use sky_types::trie::{TrieInsertError, map_base};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Trie<S: ReadWriteStorage> {
    pub(crate) root: MapBase,
    storage: S,
}
impl<S: ReadWriteStorage> StoreRead for Trie<S> {
    fn status(&self) -> StorageHead {
        self.storage.status()
    }

    fn read_root(&self) -> MapBase {
        self.root
    }
}
impl<S: ReadWriteStorage> TrieBaseRead for Trie<S> {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError> {
        self.storage.read_base(id).await
    }
}

impl<S: ReadWriteStorage> Storage<S> for Trie<S> {
    fn storage(&self) -> &S {
        &self.storage
    }
}

/// Trie construction methods.
impl<S: ReadWriteStorage> Trie<S> {
    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        let root = storage.read_root();
        Self { root, storage }
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(mut self) -> Result<Self, WriteStorageError> {
        self.storage.write_root(self.root.clone()).await?;
        Ok(self)
    }

    pub fn unwrap(self) -> MapBase {
        self.root
    }

    pub fn close(self) -> S {
        self.storage
    }

    /// A read-only view of this trie over a snapshot of its storage.
    pub fn view(&self) -> TrieReader<S::Snapshot> {
        TrieReader::new(self.root.clone(), self.storage.snapshot())
    }
}

/// Trie update methods.
impl<S: ReadWriteStorage> Trie<S> {
    pub async fn insert(mut self, key: i32, value: TrieValue) -> Result<Self, TrieInsertError> {
        let key = HashKey::new(key);
        let root = map_base::insert_kv(self.root, key, value, &mut self.storage).await?;
        self.root = root;
        Ok(self)
    }

    pub async fn deep_insert<const N: usize>(
        mut self,
        key: [i32; N],
        value: impl Into<TrieValue>,
        replace_tail: bool,
    ) -> Result<Self, TrieInsertError> {
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
                match query_value(*map_base, key, &self.storage).await? {
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
            let post_map_base =
                map_base::insert_kv(pre_map_base.clone(), key, value, &mut self.storage).await?;
            value = TrieValue::SubTrie(post_map_base);
        }
        let TrieValue::SubTrie(root) = value else {
            panic!("value should be map_base")
        };
        self.root = root;
        Ok(self)
    }
}
