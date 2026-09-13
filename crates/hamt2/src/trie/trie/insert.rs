use crate::TransactError;
use crate::trie::Trie;
use crate::trie::trie_storage::ReadWriteTrieStorage;
use crate::trie::types::hash_key::HashKey;
use crate::trie::types::trie_value::TrieValue;

impl<S: ReadWriteTrieStorage> Trie<S> {
    pub async fn insert(mut self, key: i32, value: TrieValue) -> Result<Self, TransactError> {
        let key = HashKey::new(key);
        let root = self.root.insert_kv(key, value, &mut self.storage).await?;
        self.root = root;
        Ok(self)
    }
}
