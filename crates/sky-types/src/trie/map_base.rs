use crate::trie::{SlotBaseId, SlotMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MapBase {
    pub map: SlotMap,
    pub base: SlotBaseId,
}

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: SlotBaseId(0),
        }
    }
}