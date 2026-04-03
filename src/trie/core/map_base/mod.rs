use crate::space::core::reader::SlotValue;
use crate::trie::core::map::TrieMap;
use crate::trie::mem::base::MemBase;
use crate::trie::space::map_base::SpaceMapBase;
use serde::{Deserialize, Serialize};

pub mod cons;
pub mod insert;
pub mod query;
pub mod write;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrieMapBase {
    Mem(TrieMap, MemBase),
    Space(SlotValue),
}

impl TrieMapBase {
    pub fn map(&self) -> TrieMap {
        match self {
            TrieMapBase::Mem(map, _) => map.clone(),
            TrieMapBase::Space(slot_value) => {
                let map_base = SpaceMapBase::assert(*slot_value);
                let map = map_base.to_map();
                map
            }
        }
    }
}
