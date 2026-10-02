use crate::{BufferIndex, SlotMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Default)]
pub struct MapBase {
    pub map: SlotMap,
    pub base: BufferIndex,
}

impl MapBase {
    pub fn empty() -> Self {
        Self {
            map: SlotMap::empty(),
            base: BufferIndex::ZERO,
        }
    }
}
