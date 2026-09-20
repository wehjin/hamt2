use serde::{Deserialize, Serialize};

mod store;
pub mod error;
mod file;
mod mem;
mod traits;

use crate::trie::{HandleTrieConfig, MapBase, SlotBaseId, TrieConfig};
pub use store::*;
pub use error::*;
pub use file::*;
pub use mem::*;
pub use traits::*;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(bound(
    serialize = "C::HandleType: Serialize",
    deserialize = "C::HandleType: Deserialize<'de>"
))]
pub struct StorageHead<C: TrieConfig = HandleTrieConfig> {
    pub max_id: SlotBaseId,
    pub root: MapBase<C>,
}

#[cfg(test)]
mod tests;
