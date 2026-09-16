use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SlotBaseId(pub i32);

impl SlotBaseId {
    /// The reserved id of the empty base. It is never stored.
    pub const ZERO: SlotBaseId = SlotBaseId(0);
}

impl Display for SlotBaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}
