use crate::StorageTrieQuery;
use crate::trie_storage::ReadTrieStorage;
use crate::types::trie_value::TrieValue;
use sky_types::trie::map_base::MapBase;

/// A borrowed, read-only view of a trie over a storage.
#[derive(Debug, Clone)]
pub struct TrieRef<'a, S: ReadTrieStorage> {
    root: MapBase,
    storage: &'a S,
}

impl<'a, S: ReadTrieStorage> TrieRef<'a, S> {
    pub fn new(root: MapBase, storage: &'a S) -> Self {
        Self { root, storage }
    }

    pub fn subtrie_from_value(value: TrieValue, storage: &'a S) -> Option<Self> {
        let root = match value {
            TrieValue::SubTrie(root) => root,
            TrieValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }
}

impl<'a, S: ReadTrieStorage> StorageTrieQuery<S> for TrieRef<'a, S> {
    fn root(&self) -> &MapBase {
        &self.root
    }

    fn storage(&self) -> &S {
        self.storage
    }
}
