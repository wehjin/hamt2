use crate::db::{val, Ent, Val};

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

impl From<Ent> for Dat {
    fn from(ent: Ent) -> Self {
        Dat::Ent(ent)
    }
}

impl From<u32> for Dat {
    fn from(value: u32) -> Self {
        Dat::Val(val(value))
    }
}

impl From<i32> for Dat {
    fn from(value: i32) -> Self {
        Dat::Val(val(value))
    }
}

impl From<&str> for Dat {
    fn from(value: &str) -> Self {
        Dat::Val(val(value))
    }
}

impl From<String> for Dat {
    fn from(value: String) -> Self {
        Dat::Val(val(value))
    }
}
