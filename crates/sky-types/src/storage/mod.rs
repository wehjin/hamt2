use serde::{Deserialize, Serialize};

pub mod error;
mod file;
mod mem;
mod traits;

use crate::trie::{MapBase, SlotBaseId};
pub use error::*;
pub use file::*;
pub use mem::*;
pub use traits::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct StorageHead {
    pub max_id: SlotBaseId,
    pub root: MapBase<SlotBaseId>,
}

#[cfg(test)]
mod tests;
