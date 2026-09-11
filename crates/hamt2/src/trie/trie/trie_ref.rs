use crate::trie::TrieQuery;
use crate::trie::base_storage::BaseStorageRead;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::value::MemValue;

/// A borrowed, read-only view of a trie over a storage.
#[derive(Debug, Clone)]
pub struct TrieRef<'a, S: BaseStorageRead> {
    root: MapBase,
    storage: &'a S,
}

impl<'a, S: BaseStorageRead> TrieRef<'a, S> {
    pub fn new(root: MapBase, storage: &'a S) -> Self {
        Self { root, storage }
    }

    pub fn subtrie_from_value(value: MemValue, storage: &'a S) -> Option<Self> {
        let root = match value {
            MemValue::MapBase(root) => root,
            MemValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }
}

impl<'a, S: BaseStorageRead> TrieQuery<S> for TrieRef<'a, S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        self.storage
    }
}