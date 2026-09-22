use crate::trie::{BaseId, SlotMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct MapBase {
    pub map: SlotMap,
    pub base: BaseId,
}

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: BaseId::ZERO,
        }
    }
}
