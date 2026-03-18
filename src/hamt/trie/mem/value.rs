use crate::hamt::trie::map_base::TrieMapBase;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq)]
pub enum MemValue {
    U32(u32),
    MapBase(TrieMapBase),
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
