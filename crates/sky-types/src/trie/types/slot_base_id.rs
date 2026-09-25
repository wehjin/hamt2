use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub};

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct BaseId(pub i32);

impl Add<i32> for BaseId {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<i32> for BaseId {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Add<usize> for BaseId {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as i32)
    }
}

impl Display for BaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Into<usize> for BaseId {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl BaseId {
    /// The reserved id of the empty base. It is never stored.
    pub const ZERO: BaseId = BaseId(0);
    pub const EMPTY: BaseId = BaseId(0);
}
