use crate::db::{Ent, Val};

pub fn dat(from: impl Into<Dat>) -> Dat {
    from.into()
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Dat {
    Val(Val),
    Ent(Ent),
}

impl From<Val> for Dat {
    fn from(val: Val) -> Self {
        Dat::Val(val)
    }
}
