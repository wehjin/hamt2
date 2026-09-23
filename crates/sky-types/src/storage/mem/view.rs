use crate::storage::{ReadStorageError, StorageStatus, TrieView};
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieStream};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MemTrieView {
    pub(crate) past: Option<Arc<MemTrieView>>,
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) max_id: BaseId,
    pub(crate) root: MapBase,
}

impl MemTrieView {
    pub fn empty() -> Self {
        Self {
            past: None,
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId::ZERO,
            root: MapBase::empty(),
        }
    }
    pub fn extend(past: &MemTrieView) -> Self {
        Self {
            past: Some(Arc::new(past.clone())),
            bases: Arc::new(RwLock::new(vec![])),
            max_id: past.max_id(),
            root: past.read_root(),
        }
    }
    pub async fn merge(&mut self, extension: MemTrieView) {
        //! This is a simple merge. Advanced merge would first remove the
        //! unused bases from the extension and re-assign base ids to fill
        //! the vacancies in the bases vec.
        let status = extension.past.unwrap().status();
        assert_eq!(status, self.status(), "cannot merge what was not extended");
        {
            let read = extension.bases.read().await;
            let mut write = self.bases.write().await;
            write.extend(read.iter().cloned())
        }
        self.max_id = extension.max_id;
        self.root = extension.root;
    }
}

impl TrieStream for MemTrieView {
    type Subtrie = MemTrieView;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        self.clone().with_new_root(Some(subtrie_root))
    }
}

impl TrieView for MemTrieView {
    type Snapshot = MemTrieView;

    fn status(&self) -> StorageStatus {
        StorageStatus {
            max_id: self.max_id,
            root: self.root,
        }
    }
    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        let root = if let Some(new_root) = new_root {
            new_root
        } else {
            self.root
        };
        Self { root, ..self }
    }
    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl BaseRead for MemTrieView {
    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id <= BaseId::ZERO {
            return Ok(Base::empty());
        }
        let Some(past) = &self.past else {
            // no past
            let base = if id > self.max_id {
                Base::empty()
            } else {
                self.bases.read().await[id.0 as usize].clone()
            };
            return Ok(base);
        };
        if id <= past.max_id {
            Box::pin(past.read_base(id)).await
        } else {
            let local_index = (id.0 - (past.max_id.0 + 1)) as usize;
            let bases = self.bases.read().await;
            if local_index >= bases.len() {
                Ok(Base::empty())
            } else {
                Ok(bases[local_index].clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::MemTrieView;
    use crate::trie::TrieStream;
    use futures::StreamExt;

    #[tokio::test]
    async fn stream_exists() {
        let trie = MemTrieView::empty();
        let stream = trie.u32_stream();
        let values: Vec<(i32, u32)> = stream.collect().await;
        assert_eq!(values, vec![]);
    }
}
