use crate::trie::SlotBaseId;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait TrieConfig: Clone + Debug + PartialEq + Eq {
    type HandleType: Clone + Eq + PartialEq + Default + Debug;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct HandleTrieConfig;

impl TrieConfig for HandleTrieConfig {
    type HandleType = SlotBaseId;
}
