use crate::storage::ReadStorageError;
use crate::trie::{Base, BaseId, MapBase};

#[allow(async_fn_in_trait)]
pub trait BaseRead {
    /// Get the maximum base id available for reading.
    fn max_id(&self) -> BaseId;

    /// Reads the trie's root.
    fn read_root(&self) -> MapBase;

    /// Reads a base from the trie's state at position `id`.
    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError>;
}
