use crate::trie::base::{Base, BaseId};
use thiserror::Error;

pub mod file;
pub mod mem;

#[derive(Debug, Error)]
pub enum BaseStorageReadError {
    #[error("base id {0} has not been written")]
    NotFound(BaseId),
    #[error("failed to read base id {0} from disk: {1}")]
    Io(BaseId, #[source] std::io::Error),
    #[error("failed to decode base id {0}: {1}")]
    Decode(BaseId, #[source] postcard::Error),
}

/// A trait for reading Bases from storage.
///
/// Base id 0 is reserved and always represents the empty base.
pub trait BaseStorageRead {
    /// Reads a base from storage.
    fn read(&self, id: BaseId) -> impl Future<Output = Result<Base, BaseStorageReadError>>;

    /// Returns the highest base id in the storage or none if empty. Base id 0 (the empty base) is not counted.
    fn max_id(&self) -> Option<BaseId>;
}

#[derive(Debug, Error)]
pub enum BaseStorageWriteError {
    #[error("failed to write base id {0} to disk: {1}")]
    Io(BaseId, #[source] std::io::Error),
    #[error("failed to encode base id {0}: {1}")]
    Encode(BaseId, #[source] postcard::Error),
}

/// A trait for reading and writing Bases from storage.
pub trait BaseStorageReadWrite: BaseStorageRead {
    /// Read the next available base id. The value is 1 in an empty storage because base id 0 is reserved for the empty base.
    fn next_id(&self) -> BaseId;

    /// Stores a base and assigns it the next available id. The id can be used to read back the base in `BaseStorageRead::read`.
    fn append(
        &mut self,
        base: &Base,
    ) -> impl Future<Output = Result<BaseId, BaseStorageWriteError>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::base_storage::mem::MemBaseStorage;
    use crate::trie::core::key::TrieKey;
    use crate::trie::mem::value::MemValue;

    #[tokio::test]
    async fn empty_storage_has_no_ids() {
        let storage = MemBaseStorage::new();
        assert_eq!(None, storage.max_id());
        assert_eq!(BaseId(1), storage.next_id());
    }

    #[tokio::test]
    async fn base_id_zero_is_the_empty_base() {
        let storage = MemBaseStorage::new();
        assert_eq!(Base::new(), storage.read(BaseId(0)).await.expect("read"));
    }

    #[tokio::test]
    async fn append_assigns_sequential_ids() {
        let mut storage = MemBaseStorage::new();
        let base = Base::new_kv(TrieKey::new(7), MemValue::U32(7));
        let id0 = storage.append(&base).await.expect("append");
        let id1 = storage.append(&base).await.expect("append");
        assert_eq!(BaseId(1), id0);
        assert_eq!(BaseId(2), id1);
        assert_eq!(Some(BaseId(2)), storage.max_id());
        assert_eq!(BaseId(3), storage.next_id());
        assert_eq!(base, storage.read(id0).await.expect("read"));
        assert_eq!(base, storage.read(id1).await.expect("read"));
    }
}
