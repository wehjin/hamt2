use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TableAddr(pub u32);

impl TableAddr {
    pub const ZERO: TableAddr = TableAddr(0);
    pub fn to_u32(&self) -> u32 {
        self.0
    }
    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<usize> for TableAddr {
    fn from(index: usize) -> Self {
        TableAddr(index as u32)
    }
}
impl From<u32> for TableAddr {
    fn from(index: u32) -> Self {
        TableAddr(index)
    }
}

impl fmt::Display for TableAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableAddr").field("index", &self.0).finish()
    }
}

impl Add<usize> for &TableAddr {
    type Output = TableAddr;

    fn add(self, rhs: usize) -> Self::Output {
        TableAddr(self.0 + rhs as u32)
    }
}

impl Add<usize> for TableAddr {
    type Output = TableAddr;
    fn add(self, rhs: usize) -> Self::Output {
        TableAddr(self.0 + rhs as u32)
    }
}

impl AddAssign<usize> for TableAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs as u32;
    }
}

impl Sub for TableAddr {
    type Output = usize;
    fn sub(self, rhs: Self) -> Self::Output {
        self.0 as usize - rhs.0 as usize
    }
}
