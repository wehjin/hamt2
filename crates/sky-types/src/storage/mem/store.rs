use crate::storage::traits::BaseStore;
use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{Base, BaseId, MapBase};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct MemBaseStore {
    pub(crate) start_id: BaseId,
    pub(crate) bases: Vec<Arc<Base>>,
    pub(crate) root: MapBase,
}

impl MemBaseStore {
    pub fn new() -> Self {
        let new = Self {
            start_id: BaseId::ZERO,
            bases: vec![Arc::new(Base::empty())],
            root: MapBase::empty(),
        };
        debug_assert_eq!(new.max_id(), BaseId(0));
        new
    }
    fn to_index(&self, id: BaseId) -> usize {
        (id.0 - self.start_id.0) as usize
    }
}

impl BaseStore for MemBaseStore {
    fn max_id(&self) -> BaseId {
        self.start_id + self.bases.len() - 1
    }

    fn start_id(&self) -> BaseId {
        self.start_id
    }

    fn root(&self) -> MapBase {
        self.root
    }

    async fn set_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.root = root;
        Ok(())
    }

    fn with_root(&self, new: Option<MapBase>) -> Self {
        let root = new.unwrap_or(self.root);
        Self {
            start_id: self.start_id,
            bases: self.bases.clone(),
            root,
        }
    }

    async fn base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let base = if id < self.start_id || id > self.max_id() {
            Base::empty()
        } else {
            let index = self.to_index(id);
            self.bases[index].deref().clone()
        };
        Ok(base)
    }

    async fn push_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let id = self.max_id() + 1;
        let index = self.to_index(id);
        debug_assert_eq!(index, self.bases.len());
        self.bases.push(Arc::new(base));
        Ok(id)
    }

    async fn extend(&self) -> Result<Self, WriteStorageError>
    where
        Self: Sized,
    {
        let extension = Self {
            bases: vec![],
            start_id: self.max_id() + 1,
            root: self.root(),
        };
        Ok(extension)
    }

    async fn commit(self, past: &Self) -> Result<Self, WriteStorageError>
    where
        Self: Sized,
    {
        //! This is a simple merge. Advanced merge would first remove the
        //! unused bases from the extension and re-assign base ids to fill
        //! the vacancies in the bases vec.
        assert_eq!(
            self.start_id,
            past.max_id() + 1,
            "cannot merge what was not extended"
        );
        let merged = Self {
            start_id: past.start_id,
            bases: past.bases.clone().into_iter().chain(self.bases).collect(),
            root: self.root,
        };
        Ok(merged)
    }

    fn snapshot(&self) -> Self {
        self.clone()
    }
}