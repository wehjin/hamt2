use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{Base, BaseId, MapBase};

#[allow(async_fn_in_trait)]
pub trait BaseStore {
    fn max_id(&self) -> BaseId;
    fn start_id(&self) -> BaseId;
    fn root(&self) -> MapBase;
    async fn set_root(&mut self, root: MapBase) -> Result<(), WriteStorageError>;
    fn with_root(&self, new: Option<MapBase>) -> Self;
    async fn base(&self, id: BaseId) -> Result<Base, ReadStorageError>;
    async fn push_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError>;
    async fn extend(&self) -> Result<Self, WriteStorageError>
    where
        Self: Sized;
    async fn commit(self, past: &Self) -> Result<Self, WriteStorageError>
    where
        Self: Sized;
    fn snapshot(&self) -> Self;
}
