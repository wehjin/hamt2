use crate::hamt::space::{TableAddr, ValueAddr};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Val(pub u16);

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Val").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug)]
pub enum TableItem {
    KeyValue(i32, ValueAddr),
    MapBase(u32, TableAddr),
}

#[derive(Debug)]
pub struct TableRoot(pub u32, pub TableAddr);
