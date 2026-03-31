use crate::space::Space;
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::value::MemValue;
use crate::trie::space::trie::SpaceTrie;
use crate::QueryError;

impl<T: Space> SpaceTrie<T> {
    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.map_base.query_keys_values(&self.reader).await
    }
}
