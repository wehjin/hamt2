mod deep_key;
pub mod slot;
pub mod slot_base;

pub use deep_key::*;
use serde::{Deserialize, Serialize};
pub use sky_types::trie::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageHead {
    pub max_id: SlotBaseId,
    pub root: MapBase,
}
