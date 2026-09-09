use crate::space::Space;
use crate::trie::core::map_base::TrieMapBase;
use crate::trie::space::root::SpaceRoot;
use crate::{space, TransactError};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemValue {
    U32(u32),
    MapBase(TrieMapBase),
}

impl MemValue {
    pub fn save<T: Space>(self, extend: &mut space::Extend<T>) -> Result<u32, TransactError> {
        match self {
            MemValue::U32(v) => Ok(v),
            MemValue::MapBase(map_base) => {
                let space_map_base = map_base.into_space_map_base(extend)?;
                let (map, base_addr) = space_map_base.into_map_base_addr();
                let space_root = SpaceRoot(map, base_addr);
                let root_addr = space_root.into_root_addr(extend)?;
                Ok(root_addr.to_u32())
            }
        }
    }
}

impl From<u32> for MemValue {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl Debug for MemValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MemValue::U32(v) => f
                .debug_tuple("MemValue")
                .field(&format_args!("U32[{:?}]", v))
                .finish(),
            MemValue::MapBase(_) => f
                .debug_tuple("MemValue")
                .field(&format_args!("MapBase"))
                .finish(),
        }
    }
}
