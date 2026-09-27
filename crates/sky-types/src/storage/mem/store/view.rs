use crate::storage::ReadStorageError;
use crate::storage::traits::view::StoreView;
use crate::trie::TrieWalk;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieSnap};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct MemView {
    pub(crate) bases: Arc<RwLock<Vec<Base>>>,
    pub(crate) max_id: BaseId,
    pub(crate) root: MapBase,
}

impl MemView {
    pub fn new() -> Self {
        Self {
            bases: Arc::new(RwLock::new(vec![Base::empty()])),
            max_id: BaseId::ZERO,
            root: MapBase::empty(),
        }
    }
}

impl BaseRead for MemView {
    fn max_id(&self) -> BaseId {
        self.max_id
    }

    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let base = if id < BaseId::ZERO || id > self.max_id {
            Base::empty()
        } else {
            let index = id.0 as usize;
            self.bases.read().await[index].clone()
        };
        Ok(base)
    }
}

impl TrieSnap for MemView {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }
}

impl TrieWalk for MemView {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        MemView {
            root: subtrie_root,
            ..self.clone()
        }
    }
}

impl StoreView for MemView {}
