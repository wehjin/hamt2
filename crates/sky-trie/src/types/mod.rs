mod deep_key;
pub mod slot;
pub mod slot_base;
pub use deep_key::*;
pub use sky_types::trie::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StorageHead {
    pub max_id: Option<SlotBaseId>,
    pub root: Option<MapBase>,
}
