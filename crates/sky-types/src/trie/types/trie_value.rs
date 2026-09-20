use crate::trie::{MapBase, TrieConfig};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub enum TrieValue<C: TrieConfig> {
    U32(u32),
    SubTrie(MapBase<C>),
}

impl<C: TrieConfig> From<u32> for TrieValue<C> {
    fn from(v: u32) -> Self {
        Self::U32(v)
    }
}

impl<C: TrieConfig> Debug for TrieValue<C> {
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
