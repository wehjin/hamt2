use crate::trie::{BaseCommit, HashKey, TrieInsertError, TrieValue, map_base};

impl<T: BaseCommit> TrieInsert for T {
    async fn insert(&mut self, key: i32, value: TrieValue) -> Result<&mut Self, TrieInsertError> {
        let key = HashKey::new(key);
        let root = map_base::insert_kv(self.read_root(), key, value, self).await?;
        self.commit_root(root).await?;
        Ok(self)
    }
}

#[allow(async_fn_in_trait)]
pub trait TrieInsert: BaseCommit {
    async fn insert(&mut self, key: i32, value: TrieValue) -> Result<&mut Self, TrieInsertError>;
}
