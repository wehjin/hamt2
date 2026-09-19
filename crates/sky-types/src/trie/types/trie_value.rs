use crate::trie::MapBase;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrieValue {
    U32(u32),
    SubTrie(MapBase),
}

impl From<u32> for TrieValue {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl Debug for TrieValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrieValue::U32(v) => f
                .debug_tuple("MemValue")
                .field(&format_args!("U32[{:?}]", v))
                .finish(),
            TrieValue::SubTrie(_) => f
                .debug_tuple("MemValue")
                .field(&format_args!("MapBase"))
                .finish(),
        }
    }
}
