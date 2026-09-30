use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::ops::{Add, Sub};

#[derive(
    Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub struct BufferIndex(pub i32);

impl Add<i32> for BufferIndex {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<i32> for BufferIndex {
    type Output = Self;

    fn sub(self, rhs: i32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl Add<usize> for BufferIndex {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs as i32)
    }
}

impl Display for BufferIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Into<usize> for BufferIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl BufferIndex {
    pub const ZERO: BufferIndex = BufferIndex(0);
    pub const NIL: BufferIndex = BufferIndex(-1);
}
