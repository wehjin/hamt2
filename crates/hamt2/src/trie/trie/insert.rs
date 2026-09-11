use crate::trie::base_storage::BaseStorageReadWrite;
use crate::TransactError;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::value::MemValue;
use crate::trie::Trie;

impl<S: BaseStorageReadWrite> Trie<S> {
    pub async fn insert(mut self, key: i32, value: MemValue) -> Result<Self, TransactError> {
        let key = TrieKey::new(key);
        let root = self.root.insert_kv(key, value, &mut self.storage).await?;
        self.root = root;
        Ok(self)
    }
}
