use crate::{Base, BufferIndex, MapBase, Slot};

#[allow(async_fn_in_trait)]
pub trait Buffer {
    /// Get the maximum base id available for reading.
    fn max_index(&self) -> BufferIndex;

    /// Read the next available base id.
    fn next_index(&self) -> BufferIndex {
        self.max_index() + 1
    }

    /// Reads the trie's root.
    fn get_root(&self) -> MapBase;

    /// Reads a base with `size` slots from the trie's buffer
    /// at position `id`.
    async fn get_base(&self, id: BufferIndex, size: usize) -> Base;

    /// Reads a subtrie.
    async fn get_subtrie(&self, id: BufferIndex) -> MapBase {
        let mut base = self.get_base(id, 1).await;
        let slot = base.slots.pop().expect("slot not found");
        let Slot::MapBase(map_base) = slot else {
            panic!("subtrie id should have a map-base")
        };
        map_base
    }
}
