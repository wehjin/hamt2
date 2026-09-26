use crate::storage::{ReadStorageError, StoreRead};
use crate::trie::{Base, BaseId, BaseRead, BaseView, MapBase};
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use crate::storage::read::MemRead;
    use crate::trie::{Base, BaseId, BaseRead, BaseView, MapBase};

    #[tokio::test]
    async fn is_store_read() {
        let mem = MemRead::new();
        // BaseRead
        let max_id = mem.max_id();
        assert_eq!(max_id, BaseId::ZERO);
        let root = mem.read_root();
        assert_eq!(root, MapBase::empty());
        let base = mem.read_base(BaseId::ZERO).await.unwrap();
        assert_eq!(base, Base::empty());
        // BaseView
        let snap = mem.snapshot();
        assert_eq!(snap.max_id(), max_id);
        assert_eq!(snap.read_root(), root);
        assert_eq!(snap.read_base(BaseId::ZERO).await.unwrap(), base);
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

impl BaseView for MemRead {
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
