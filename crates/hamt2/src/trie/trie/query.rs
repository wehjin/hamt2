use crate::trie::base_storage::BaseStorageReadWrite;
use crate::trie::core::key::TrieKey;
use crate::trie::mem::value::MemValue;
use crate::trie::Trie;
use crate::QueryError;
use futures::Stream;
use std::sync::Arc;

impl<S: BaseStorageReadWrite> Trie<S> {
    pub async fn query_value(&self, key: i32) -> Result<Option<MemValue>, QueryError> {
        let key = TrieKey::new(key);
        self.root.query_value(key, &self.storage).await
    }

    pub async fn query_keys_values(&self) -> Result<Vec<(i32, MemValue)>, QueryError> {
        self.root.query_keys_values(&self.storage).await
    }

    pub fn filter_map<V, Fut: Future<Output = Option<V>>>(
        &self,
        filter: impl Fn((i32, MemValue)) -> Fut,
    ) -> impl Stream<Item = V> {
        use futures::stream::StreamExt;
        let filter = Arc::new(filter);
        let stream = self.root.kv_stream(&self.storage);
        stream.filter_map(move |kv| {
            let filter = filter.clone();
            async move {
                let output = filter(kv).await;
                output
            }
        })
    }
    pub fn u32_stream(&self) -> impl Stream<Item = (i32, u32)> {
        self.filter_map(|(key, value)| async move {
            if let MemValue::U32(val) = value {
                Some((key, val))
            } else {
                None
            }
        })
    }

    pub fn subtrie_stream(&self) -> impl Stream<Item = (i32, Trie<S>)>
    where
        S: Clone,
    {
        let clone_storage: Arc<S> = Arc::new(self.storage.clone());
        self.filter_map(move |(key, value)| {
            let storage_source = clone_storage.clone();
            async move {
                Self::subtrie_from_value(value, (*storage_source).clone()).map(|subtrie| (key, subtrie))
            }
        })
    }
}
