use crate::storage::{ReadStorage, ReadStorageError, StorageStatus};
use crate::trie::{Base, BaseId, MapBase, RootBaseRead, TrieStream};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct MemReadStorage {
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) status: StorageStatus,
}

impl MemReadStorage {
    pub fn empty() -> Self {
        let bases = Arc::new(RwLock::new(vec![Base::empty()]));
        let status = StorageStatus::default();
        Self { bases, status }
    }
}

impl TrieStream for MemReadStorage {
    type Subtrie = MemReadStorage;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl ReadStorage for MemReadStorage {
    type Snapshot = MemReadStorage;

    fn status(&self) -> StorageStatus {
        self.status
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let status = self.status.with_new_root(new_root);
        Self { status, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl RootBaseRead for MemReadStorage {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        assert!(id <= self.max_id(), "id out of bounds {id:?}");
        let index = id.0 as usize;
        let read = self.bases.read().unwrap();
        let base = read[index].clone();
        Ok(base)
    }

    fn read_root(&self) -> MapBase {
        self.status().root
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::MemReadStorage;
    use crate::trie::TrieStream;
    use futures::StreamExt;

    #[tokio::test]
    async fn stream_exists() {
        let trie = MemReadStorage::empty();
        let stream = trie.u32_stream();
        let values: Vec<(i32, u32)> = stream.collect().await;
        assert_eq!(values, vec![]);
    }
}
