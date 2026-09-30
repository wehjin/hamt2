use crate::storage::ReadStorageError;
use crate::trie::{Base, BufferIndex, MapBase};

#[allow(async_fn_in_trait)]
pub trait Buffer {
    /// Get the maximum base id available for reading.
    fn max_index(&self) -> BufferIndex;

    /// Read the next available base id.
    fn next_index(&self) -> BufferIndex {
        self.max_index() + 1
    }

    /// Reads the trie's root.
    fn read_root(&self) -> MapBase;

    /// Reads a base from the trie's state at position `id`.
    async fn read_base(&self, id: BufferIndex) -> Result<Base, ReadStorageError>;
}
