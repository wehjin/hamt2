use crate::trie::{BufferIndex, MapBase};
use serde::{Deserialize, Serialize};

mod mem;
mod traits;

pub use mem::*;
pub use traits::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct StorageStatus {
    pub max_id: BufferIndex,
    pub root: MapBase,
}

impl StorageStatus {
    pub fn with_new_root(self, root: Option<MapBase>) -> Self {
        match root {
            None => self,
            Some(root) => Self { root, ..self },
        }
    }
}

#[cfg(test)]
mod tests;
