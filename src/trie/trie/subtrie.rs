use crate::QueryError;
use crate::space::Space;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::space::root::SpaceRoot;
use crate::trie::SpaceTrie;

impl<T: Space> SpaceTrie<T> {
    pub async fn to_subtrie_from_value(&self, value: MemValue) -> Result<Self, QueryError> {
        let map_base = match value {
            MemValue::MapBase(map_base) => map_base,
            MemValue::U32(u32) => SpaceRoot::from_root_addr(u32, &self.reader)
                .await?
                .into_trie_map_base(),
        };
        let subtrie = self.to_subtrie(map_base);
        Ok(subtrie)
    }

    pub fn to_subtrie(&self, map_base: TrieMapBase) -> Self {
        Self {
            map_base,
            reader: self.reader.clone(),
        }
    }
}