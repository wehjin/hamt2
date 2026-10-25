use crate::{BufferIndex, MapBase, Slot};

pub trait Buffer {
    fn get_root(&self) -> MapBase;

    /// Reads a base with `size` slots from the trie's buffer
    /// at position `id`.
    fn get_base(&self, id: BufferIndex, size: usize) -> &[Slot];

    /// Reads the bytes stored at `index`.
    fn get_bytes(&self, index: BufferIndex) -> Vec<u8>;

    /// Reads a subtrie.
    fn get_subtrie(&self, id: BufferIndex) -> MapBase {
        let base = self.get_base(id, 1);
        let slot = base.first().expect("slot not found");
        let Slot::MapBase(map_base) = slot else {
            panic!("subtrie id should have a map-base")
        };
        map_base.clone()
    }
}
