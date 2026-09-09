use crate::space::{Read, Space};
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::space::root::SpaceRoot;
use crate::{QueryError, TransactError};

pub mod deep;
pub mod insert;
pub mod query;
pub mod subtrie;

#[derive(Debug)]
pub struct SpaceTrie<T: Space> {
    map_base: TrieMapBase,
    reader: T::Reader,
}

impl<T: Space> Clone for SpaceTrie<T> {
    fn clone(&self) -> Self {
        let map_base = self.map_base.clone();
        let reader = self.reader.clone();
        Self { map_base, reader }
    }
}

impl<T: Space> SpaceTrie<T> {
    pub async fn connect(space: &T) -> Result<Self, QueryError> {
        let reader = space.read().await?;
        let map_base = match reader.read_root().await? {
            None => TrieMapBase::empty(),
            Some(root) => {
                let space_root = SpaceRoot::from_root_addr(root.to_u32(), &reader).await?;
                let trie_map_base = space_root.into_trie_map_base();
                trie_map_base
            }
        };
        let trie = Self { map_base, reader };
        Ok(trie)
    }
    pub async fn commit(self, space: &mut T) -> Result<(), TransactError> {
        let mut extend = space.extend().await?;
        let root_addr = {
            let space_map_base = self.map_base.into_space_map_base(&mut extend)?;
            let (map, base_addr) = space_map_base.into_map_base_addr();
            let space_root = SpaceRoot(map, base_addr);
            let root_addr = space_root.into_root_addr(&mut extend)?;
            root_addr
        };
        extend.set_root(root_addr);
        extend.commit(space).await
    }
    pub fn unwrap(self) -> TrieMapBase {
        self.map_base
    }
}
