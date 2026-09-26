use crate::storage::{MemView, ReadStorageError, WriteStorageError};
use crate::trie::{Base, BaseCommit, BaseId, BaseRead, MapBase};

#[cfg(test)]
mod tests {
    use crate::storage::mem::MemEdit;
    use crate::storage::mem::MemView;
    use crate::trie::{TrieInsert, TrieQuery, TrieSnap, TrieValue};

    #[tokio::test]
    async fn mem_edit_inserts() {
        let insert_value = TrieValue::U32(34);
        let view = MemView::new();
        let mut edit = MemEdit::extend(view.snapshot());
        edit.insert(33, insert_value).await.unwrap();
        let query_value = edit.query(33).await.unwrap().unwrap();
        assert_eq!(query_value, insert_value)
    }
}

/// Deliberately non-Clone.
#[derive(Debug)]
pub struct MemEdit {
    pub(crate) past: MemView,
    pub(crate) bases: Vec<Base>,
    pub(crate) root: MapBase,
}

impl MemEdit {
    pub fn extend(past: MemView) -> Self {
        let root = past.read_root();
        Self {
            past,
            bases: vec![],
            root,
        }
    }
    fn start_id(&self) -> BaseId {
        self.past.max_id() + 1
    }
}

impl BaseRead for MemEdit {
    fn max_id(&self) -> BaseId {
        self.past.max_id() + self.bases.len()
    }

    fn read_root(&self) -> MapBase {
        self.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        let start_id = self.start_id();
        if id < start_id {
            return self.past.read_base(id).await;
        }
        if id <= self.max_id() {
            let index = (id.0 - start_id.0) as usize;
            let base = self.bases[index].clone();
            Ok(base)
        } else {
            Ok(Base::empty())
        }
    }
}

impl BaseCommit for MemEdit {
    async fn commit_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.root = root;
        Ok(())
    }

    async fn commit_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError> {
        let next_id = self.next_id();
        self.bases.push(base);
        debug_assert_eq!(self.max_id(), next_id);
        Ok(next_id)
    }
}
