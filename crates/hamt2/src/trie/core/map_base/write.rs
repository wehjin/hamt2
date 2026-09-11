use crate::space::Space;
use crate::trie::base_storage::BaseStorageRead;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::space::map_base::SpaceMapBase;
use crate::{space, TransactError};

impl TrieMapBase {
    pub async fn into_space_map_base<T: Space>(
        self,
        extend: &mut space::Extend<T>,
        storage: &impl BaseStorageRead,
    ) -> Result<SpaceMapBase, TransactError> {
        let TrieMapBase { map, base } = self;
        let base = storage.read(base).await.expect("read base");
        SpaceMapBase::save(extend, map, base, storage).await
    }
}
