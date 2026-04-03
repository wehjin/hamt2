use crate::space::Space;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::mem::value::MemValue;
use crate::trie::space::root::SpaceRoot;
use crate::trie::SpaceTrie;
use crate::QueryError;

impl<T: Space> SpaceTrie<T> {
    pub async fn to_subtrie_from_value(&self, value: MemValue) -> Result<Self, QueryError> {
        Self::subtrie_from_value(value, self.reader.clone()).await
    }

    pub async fn subtrie_from_value(
        value: MemValue,
        reader: T::Reader,
    ) -> Result<Self, QueryError> {
        let map_base = match value {
            MemValue::MapBase(map_base) => map_base,
            MemValue::U32(u32) => SpaceRoot::from_root_addr(u32, &reader)
                .await?
                .into_trie_map_base(),
        };
        Ok(Self { map_base, reader })
    }

    pub fn new_subtrie(&self) -> Self {
        let map_base = TrieMapBase::empty();
        Self {
            map_base,
            reader: self.reader.clone(),
        }
    }
}
