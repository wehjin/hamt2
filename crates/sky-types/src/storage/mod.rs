use serde::{Deserialize, Serialize};

pub mod error;
mod file;
mod mem;
mod store;
mod traits;

use crate::trie::{HandleTrieConfig, MapBase, TrieConfig};
pub use error::*;
pub use file::*;
pub use mem::*;
pub use store::*;
pub use traits::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub struct StorageHead<C: TrieConfig = HandleTrieConfig> {
    pub max_id: C::HandleType,
    pub root: MapBase<C>,
}

#[cfg(test)]
mod tests;
