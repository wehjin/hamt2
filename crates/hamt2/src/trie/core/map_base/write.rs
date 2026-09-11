use crate::{space, TransactError};
use crate::space::Space;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::space::map_base::SpaceMapBase;

impl TrieMapBase {
    pub fn into_space_map_base<T: Space>(
        self,
        extend: &mut space::Extend<T>,
    ) -> Result<SpaceMapBase, TransactError> {
        let TrieMapBase { map, base } = self;
        SpaceMapBase::save(extend, map, base)
    }
}