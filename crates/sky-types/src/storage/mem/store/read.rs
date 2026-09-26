use crate::storage::{ReadStorageError, StoreRead};
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieSnap};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use crate::storage::read::MemRead;
    use crate::trie::{TrieQuery, TrieSnap, TrieStream};
    use futures::StreamExt;

    #[tokio::test]
    async fn is_query() {
        let mem = MemRead::new();
        let values = mem.query_all().await.unwrap();
        assert_eq!(values, vec![]);
    }

    #[tokio::test]
    async fn is_snap() {
        let mem = MemRead::new();
        let values = mem.snapshot().query_all().await.unwrap();
        assert_eq!(values, vec![]);
    }

    #[tokio::test]
    async fn is_stream() {
        let mem = MemRead::new();
        let values = mem.u32_stream().collect::<Vec<_>>().await;
        assert_eq!(values, vec![]);
    }
}

#[derive(Debug, Clone)]
pub struct MemRead {
    pub(crate) bases: Arc<Vec<Base>>,
    pub(crate) root: MapBase,
}

impl MemRead {
    pub fn new() -> Self {
        Self {
            bases: Arc::new(vec![Base::empty()]),
            root: MapBase::empty(),
        }
    }
}

impl BaseRead for MemRead {
    fn max_id(&self) -> BaseId {
        BaseId(self.bases.len() as i32 - 1)
    }

    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let base = if id < BaseId::ZERO || id > self.max_id() {
            Base::empty()
        } else {
            let index = id.0 as usize;
            self.bases[index].clone()
        };
        Ok(base)
    }
}

impl TrieSnap for MemRead {
    type Snapshot = Self;

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let root = new_root.unwrap_or(self.root);
        Self { root, ..self }
    }

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl StoreRead for MemRead {}
