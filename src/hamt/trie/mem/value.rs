use crate::hamt::trie::core::map_base::TrieMapBase;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemValue {
    U32(u32),
    String(String),
    MapBase(TrieMapBase),
}
impl From<u32> for MemValue {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}
impl From<String> for MemValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl Debug for MemValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MemValue::U32(v) => f
                .debug_tuple("MemValue")
                .field(&format_args!("U32[{:?}]", v))
                .finish(),
            MemValue::String(v) => f
                .debug_tuple("MemValue")
                .field(&format_args!("String[{}]", v))
                .finish(),
            MemValue::MapBase(_) => f
                .debug_tuple("MemValue")
                .field(&format_args!("MapBase"))
                .finish(),
        }
    }
}
