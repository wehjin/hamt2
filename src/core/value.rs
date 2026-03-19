use crate::hamt::space::TableAddr;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Value {
    U32(u32),
    U64(u64),
    MapBase(u32, TableAddr),
}
