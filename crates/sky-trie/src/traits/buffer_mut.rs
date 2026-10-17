use crate::{Base, Buffer, BufferIndex, MapBase, Slot};
#[allow(async_fn_in_trait)]
pub trait BufferMut: Buffer
where
    Self: Sized,
{
    /// Commits a new `root` into the trie.
    async fn push_root(&mut self, root: MapBase);

    /// Commits a base and returns its assigned handle.
    async fn push_base(&mut self, base: Base) -> BufferIndex;

    /// Commits a subtrie.
    async fn push_subtrie(&mut self, map_base: MapBase) -> BufferIndex {
        let slot = Slot::MapBase(map_base);
        let base = Base { slots: vec![slot] };
        self.push_base(base).await
    }
}
