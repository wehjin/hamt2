use crate::storage::{ReadStorageError, WriteStorageError};
use crate::trie::{Base, BaseId, MapBase};

pub mod edit;
pub mod load;
pub mod view;

#[allow(async_fn_in_trait)]
pub trait BaseStore {
    fn max_id(&self) -> BaseId;
    fn read_root(&self) -> MapBase;
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError>;

    fn start_id(&self) -> BaseId;
    async fn set_root(&mut self, root: MapBase) -> Result<(), WriteStorageError>;
    fn with_root(&self, new: Option<MapBase>) -> Self;
    async fn push_base(&mut self, base: Base) -> Result<BaseId, WriteStorageError>;
    async fn extend(&self) -> Result<Self, WriteStorageError>
    where
        Self: Sized;
    async fn commit(self, past: &Self) -> Result<Self, WriteStorageError>
    where
        Self: Sized;
    fn snapshot(&self) -> Self;
}
