use crate::hamt::trie::core::map_base::TrieMapBase;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum MemValue {
    U32(u32),
    MapBase(TrieMapBase),
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
