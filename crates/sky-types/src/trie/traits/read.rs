use crate::storage::ReadStorageError;
use crate::trie::{MapBase, SlotBase, SlotBaseId};
#[allow(async_fn_in_trait)]
pub trait TrieRead {
    async fn read_base(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError>;
    fn read_root(&self) -> MapBase;
}
