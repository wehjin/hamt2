use crate::trie::Trie;
use crate::trie::base_storage::errors::BaseStorageReadError;
use crate::trie::base_storage::{BaseStorageRead, BaseStorageReadWrite};
use crate::trie::core::map_base::MapBase;

#[derive(Debug)]
pub struct ReadTrie<S: BaseStorageRead> {
    root: MapBase,
    storage: S,
}

impl<S: BaseStorageReadWrite> ReadTrie<S> {
    pub async fn connect(storage: S) -> Result<Self, BaseStorageReadError> {
        todo!()
    }
}
