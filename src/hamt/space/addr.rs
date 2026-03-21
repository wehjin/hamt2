use crate::hamt::space::value::Val;
use crate::hamt::space::seg::Seg;
use std::fmt;
use crate::hamt::space::table::TablePos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Addr {
    Value(ValueAddr),
    Table(TableAddr),
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Addr").finish()
    }
}

impl From<TableAddr> for Addr {
    fn from(addr: TableAddr) -> Self {
        Addr::Table(addr)
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
