pub mod error;

use crate::trie::{MapBase, SlotBaseId};
pub use error::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct StorageHead {
    pub max_id: SlotBaseId,
    pub root: MapBase,
}
