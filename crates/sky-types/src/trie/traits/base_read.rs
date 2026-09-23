use crate::storage::ReadStorageError;
use crate::trie::{Base, BaseId, MapBase};
#[allow(async_fn_in_trait)]
pub trait BaseRead {
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError>;
    fn read_root(&self) -> MapBase;
}
