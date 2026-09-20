use crate::trie::SlotMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct MapBase<HandleType> {
    pub map: SlotMap,
    pub base: HandleType,
}

impl<HandleType: Default> MapBase<HandleType> {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: HandleType::default(),
        }
    }
}
