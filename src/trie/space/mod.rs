use crate::trie::core::map::TrieMap;
use crate::trie::core::map_base::TrieMapBase;

use crate::space::Space;
use crate::space::{Read, TableAddr};
use crate::trie::space::map_base::SpaceMapBase;
use crate::TransactError;
use crate::{space, QueryError};
pub mod map_base;
pub mod slots;
pub mod trie;

pub struct SpaceRoot(pub TrieMap, pub TableAddr);
impl SpaceRoot {
    pub fn into_root_addr<T: Space>(
        self,
        extend: &mut space::Extend<T>,
    ) -> Result<TableAddr, TransactError> {
        let slot_value = SpaceMapBase::new(self.0, self.1).into_slot_value();
        let root_addr = extend.add_slots(vec![slot_value]);
        Ok(root_addr)
    }
    pub async fn from_root_addr(root_addr: u32, reader: &impl Read) -> Result<Self, QueryError> {
        debug_assert_eq!(root_addr & 0x8000_0000, 0);
        let slot_value = reader.read_slot(&TableAddr(root_addr), 0).await?;
        let (key, addr) = SpaceMapBase::assert(slot_value).into_map_base_addr();
        Ok(Self(key, addr))
    }
    pub fn into_trie_map_base(self) -> TrieMapBase {
        SpaceMapBase::new(self.0, self.1).into_trie_map_base()
    }
    pub fn from_trie_map_base<T: Space>(
        trie_map_base: TrieMapBase,
        extend: &mut space::Extend<T>,
    ) -> Result<Self, TransactError> {
        let space_map_base = trie_map_base.into_space_map_base(extend)?;
        let (key, addr) = space_map_base.into_map_base_addr();
        Ok(Self(key, addr))
    }
}
