use crate::hamt::space::core::{TablePos, Val};
use crate::hamt::space::seg::Seg;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Addr {
    Value(Seg, Val),
    Table(Seg, TablePos),
}

impl Addr {
    pub fn offset_table(self, offset: usize) -> Self {
        let Addr::Table(seg, pos) = self else {
            panic!("Cannot offset a non-table address")
        };
        Self::Table(seg, pos + offset)
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
