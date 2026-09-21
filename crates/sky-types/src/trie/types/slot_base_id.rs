use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::ops::Add;

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct SlotBaseId(pub i32);

impl Add<i32> for SlotBaseId {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Display for SlotBaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Into<usize> for SlotBaseId {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl SlotBaseId {
    /// The reserved id of the empty base. It is never stored.
    pub const ZERO: SlotBaseId = SlotBaseId(0);
}
