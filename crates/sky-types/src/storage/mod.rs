use serde::{Deserialize, Serialize};

pub mod error;
mod file;
mod mem;
mod store;
mod traits;

use crate::trie::{MapBase, SlotBaseId};
pub use error::*;
pub use file::*;
pub use mem::*;
pub use store::*;
pub use traits::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct StorageHead {
    pub max_id: SlotBaseId,
    pub root: MapBase,
}

impl StorageHead {
    pub fn with_new_root(self, root: Option<MapBase>) -> Self {
        match root {
            None => self,
            Some(root) => Self { root, ..self },
        }
    }
}

#[cfg(test)]
mod tests;
