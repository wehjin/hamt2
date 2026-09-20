use crate::trie::{SlotMap, TrieConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub struct MapBase<C: TrieConfig> {
    pub map: SlotMap,
    pub base: C::HandleType,
}

impl<C: TrieConfig> MapBase<C> {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: C::HandleType::default(),
        }
    }
}
