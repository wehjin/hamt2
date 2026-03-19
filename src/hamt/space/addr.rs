use crate::hamt::space::core::{TablePos, Val};
use crate::hamt::space::seg::Seg;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Addr {
    Value(ValueAddr),
    Table(TableAddr),
}

impl Addr {
    pub fn offset_table(self, offset: usize) -> Self {
        match self {
            Addr::Value(_) => {
                panic!("Cannot offset a non-table address")
            }
            Addr::Table(table_addr) => Addr::Table(table_addr.offset(offset)),
        }
    }
}

impl std::fmt::Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Addr").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ValueAddr(pub Seg, pub Val);

impl fmt::Display for ValueAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueAddr")
            .field("seg", &self.0)
            .field("val", &self.1)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TableAddr(pub Seg, pub TablePos);

impl fmt::Display for TableAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableAddr")
            .field("seg", &self.0)
            .field("pos", &self.1)
            .finish()
    }
}

impl TableAddr {
    pub fn offset(self, offset: usize) -> Self {
        let TableAddr(seg, pos) = self;
        TableAddr(seg, pos + offset)
    }
}
