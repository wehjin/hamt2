use crate::hamt::space::seg::Seg;
use crate::hamt::space::val::Val;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Addr {
    Value(Seg, Val),
    Slots(Seg),
}

impl std::fmt::Display for Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Addr").finish()
    }
}
