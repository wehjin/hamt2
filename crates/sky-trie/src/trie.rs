use crate::TrieReader;
use crate::prelude::TrieValue;
use sky_types::storage::error::WriteStorageError;
use sky_types::storage::{ReadStorage, ReadStorageError, ReadWriteStorage, StorageStatus};
use sky_types::trie::TrieStream;
use sky_types::trie::map_base::query_value;
use sky_types::trie::{Base, BaseId, DeepKey, HashKey, MapBase, RootBaseRead};
use sky_types::trie::{TrieInsertError, map_base};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Trie<S: ReadWriteStorage> {
    pub(crate) storage: S,
}

impl<S: ReadWriteStorage> TrieStream for Trie<S> {
    type Subtrie = TrieReader<S::Snapshot>;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.snapshot().to_subtrie(subtrie_root)
    }
}
impl<S: ReadWriteStorage> ReadStorage for Trie<S> {
    type Snapshot = TrieReader<S::Snapshot>;

    fn status(&self) -> StorageStatus {
        self.storage.status()
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let storage = self.storage.with_new_root(new_root);
        Self { storage }
    }

    fn snapshot(&self) -> Self::Snapshot {
        let snap_storage = self.storage.snapshot();
        TrieReader::new(snap_storage)
    }
}
impl<S: ReadWriteStorage> RootBaseRead for Trie<S> {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        self.storage.read_base(id).await
    }

    fn read_root(&self) -> MapBase {
        self.storage.read_root()
    }
}

/// Trie construction methods.
impl<S: ReadWriteStorage> Trie<S> {
    /// Connects to the storage, loading the persisted root.
    pub fn connect(storage: S) -> Self {
        Self { storage }
    }

    /// Persists the current root map base to the storage.
    pub async fn commit(self) -> Result<Self, WriteStorageError> {
        // We're writing directly as we go along so there is nothing
        // to do here for now. We can do better by not writing directly
        // to support rewind and compaction.
        Ok(self)
    }

    pub fn close(self) -> S {
        self.storage
    }
}

/// Trie update methods.
impl<S: ReadWriteStorage> Trie<S> {
    pub async fn insert(mut self, key: i32, value: TrieValue) -> Result<Self, TrieInsertError> {
        let key = HashKey::new(key);
        let root = map_base::insert_kv(self.read_root(), key, value, &mut self.storage).await?;
        self.storage.commit_root(root).await?;
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
        map_bases.insert(0, self.read_root().clone());
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
        self.storage.commit_root(root).await?;
        Ok(self)
    }
}
