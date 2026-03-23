use crate::space::{TableAddr, ValueAddr};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug)]
pub enum TableItem {
    KeyValue(i32, ValueAddr),
    MapBase(u32, TableAddr),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableRoot(pub u32, pub TableAddr);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TablePos(pub u32);

impl std::fmt::Display for TablePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TablePos").finish()
    }
}

impl Add<usize> for TablePos {
    type Output = TablePos;
    fn add(self, rhs: usize) -> TablePos {
        TablePos(self.0 + rhs as u32)
    }
}
