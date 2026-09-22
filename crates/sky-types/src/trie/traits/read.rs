use crate::trie::{MapBase, SlotBase, SlotBaseId, TrieQueryError};
#[allow(async_fn_in_trait)]
pub trait TrieRead {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, TrieQueryError>;
    fn read_root(&self) -> MapBase;
}
