use crate::{Base, BufferIndex, MapBase, Slot};

pub trait Buffer {
    /// Get the maximum base id available for reading.
    fn max_index(&self) -> BufferIndex;

    /// Read the next available base id.
    fn next_index(&self) -> BufferIndex {
        self.max_index() + 1
    }

    /// Reads a base with `size` slots from the trie's buffer
    /// at position `id`.
    fn get_base(&self, id: BufferIndex, size: usize) -> Base;

    /// Reads a subtrie.
    fn get_subtrie(&self, id: BufferIndex) -> MapBase {
        let mut base = self.get_base(id, 1);
        let slot = base.slots.pop().expect("slot not found");
        let Slot::MapBase(map_base) = slot else {
            panic!("subtrie id should have a map-base")
        };
        map_base
    }
}
