use crate::trie::MapBase;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrieValue<H> {
    U32(u32),
    SubTrie(MapBase<H>),
}

impl<H> From<u32> for TrieValue<H> {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl<H> Debug for TrieValue<H> {
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
