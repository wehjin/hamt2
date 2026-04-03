use crate::space::Space;
use crate::trie::core::key::TrieKey;
use crate::trie::core::query::QueryKeysValues;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;
use crate::QueryError;

impl<T: Space> SpaceTrie<T> {
    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.map_base.query_keys_values(&self.reader).await
    }
}

impl<T: Space> SpaceTrie<T> {
    pub async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        self.map_base.query_value(key, &self.reader).await
    }
}
