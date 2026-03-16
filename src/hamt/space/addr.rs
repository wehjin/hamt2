use crate::hamt::space::core::{TablePos, Val};
use crate::hamt::space::seg::Seg;

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
