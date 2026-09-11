use crate::space::Space;
use crate::TransactError;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::value::MemValue;
use crate::trie::SpaceTrie;

impl<T: Space> SpaceTrie<T> {
    pub async fn insert(mut self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let map_base = self.map_base.insert_kv(key, value, &mut self.storage).await?;
        self.map_base = map_base;
        Ok(self)
    }
}
