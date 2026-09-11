use crate::trie::base::{Base, BaseId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaseStorageReadError {}

/// A trait for reading Bases from storage.
pub trait BaseStorageRead {
    /// Reads a base from storage.
    fn read(&self, id: BaseId) -> impl Future<Output = Result<Base, BaseStorageReadError>>;

    /// Returns the highest base id in the storage or none if empty.
    fn max_id(&self) -> Option<BaseId>;
}

#[derive(Debug, Error)]
pub enum BaseStorageWriteError {}

/// A trait for reading and writing Bases from storage.
pub trait BaseStorageReadWrite: BaseStorageRead {
    /// Read the next available base id. The value is 0 in an empty storage and grows with each append.
    fn next_id(&self) -> BaseId;

    /// Stores a base and assigns it the next available id. The id can be used to read back the base in `BaseStorageRead::read`.
    fn append(
        &mut self,
        base: &Base,
    ) -> impl Future<Output = Result<BaseId, BaseStorageWriteError>>;
}

/// An in-memory storage for Bases backed by a `Vec<Base>`.
#[derive(Debug, Clone, Default)]
pub struct MemBaseStorage {
    bases: Vec<Base>,
}

impl MemBaseStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BaseStorageRead for MemBaseStorage {
    async fn read(&self, id: BaseId) -> Result<Base, BaseStorageReadError> {
        let index = id.0 as usize;
        let base = self.bases[index].clone();
        Ok(base)
    }

    fn max_id(&self) -> Option<BaseId> {
        let len = self.bases.len();
        if len == 0 {
            None
        } else {
            Some(BaseId((len - 1) as i32))
        }
    }
}

impl BaseStorageReadWrite for MemBaseStorage {
    fn next_id(&self) -> BaseId {
        BaseId(self.bases.len() as i32)
    }

    async fn append(&mut self, base: &Base) -> Result<BaseId, BaseStorageWriteError> {
        let id = self.next_id();
        self.bases.push(base.clone());
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::core::key::TrieKey;
    use crate::trie::mem::value::MemValue;

    #[tokio::test]
    async fn empty_storage_has_no_ids() {
        let storage = MemBaseStorage::new();
        assert_eq!(None, storage.max_id());
        assert_eq!(BaseId(0), storage.next_id());
    }

    #[tokio::test]
    async fn append_assigns_sequential_ids() {
        let mut storage = MemBaseStorage::new();
        let base = Base::new_kv(TrieKey::new(7), MemValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = storage.append(&base).await.expect("append");
        assert_eq!(BaseId(0), id0);
        assert_eq!(BaseId(1), id1);
        assert_eq!(Some(BaseId(1)), storage.max_id());
        assert_eq!(BaseId(2), storage.next_id());
        assert_eq!(base, storage.read(id0).await.expect("read"));
        assert_eq!(base, storage.read(id1).await.expect("read"));
    }
}
