use serde::{Deserialize, Serialize};

pub mod error;

#[cfg(feature = "fs")]
mod file;
mod mem;
mod traits;
mod trie_edit;
mod trie_load;
mod trie_view;

use crate::trie::{BaseId, MapBase};
pub use error::*;
#[cfg(feature = "fs")]
pub use file::*;
pub use mem::*;
pub use traits::*;
pub use trie_edit::*;
pub use trie_load::*;
pub use trie_view::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct StorageStatus {
    pub max_id: BaseId,
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
