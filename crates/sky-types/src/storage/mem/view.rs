use crate::storage::ReadStorageError;
use crate::storage::traits::view::StoreView;
use crate::trie::TrieWalk;
use crate::trie::{Base, BaseId, BaseRead, MapBase, TrieSnap};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

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

impl Eq for MemView {}

impl PartialEq for MemView {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
            && self.max_id == other.max_id
            && self.bases.deref().read().unwrap().deref()
                == other.bases.deref().read().unwrap().deref()
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
            self.bases.read().unwrap()[index].clone()
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
