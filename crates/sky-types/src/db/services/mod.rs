use crate::db::{Dat, Ein, Ent, Val};

pub mod datom;

pub fn val(from: impl Into<Val>) -> Val {
    from.into()
}

pub fn dat(from: impl Into<Dat>) -> Dat {
    from.into()
}

pub fn ein(from: impl Into<Ein>) -> Ein {
    from.into()
}

pub fn ent(ent: impl Into<Ent>) -> Ent {
    ent.into()
}