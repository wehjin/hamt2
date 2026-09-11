use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::map_base::MapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::Trie;

impl<S: BaseStorageReadWrite> Trie<S> {
    /// Converts a map-base value into a sub-trie over the same storage.
    pub fn to_subtrie_from_value(&self, value: MemValue) -> Option<Self>
    where
        S: Clone,
    {
        Self::subtrie_from_value(value, self.storage.clone())
    }

    pub fn subtrie_from_value(value: MemValue, storage: S) -> Option<Self> {
        let root = match value {
            MemValue::MapBase(root) => root,
            MemValue::U32(_) => return None,
        };
        Some(Self { root, storage })
    }

    pub fn new_subtrie(&self) -> Self
    where
        S: Clone,
    {
        let root = MapBase::empty();
        Self {
            root,
            storage: self.storage.clone(),
        }
    }
}
