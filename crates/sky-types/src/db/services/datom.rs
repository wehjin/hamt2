use crate::db::types::{Datom, Dir};
use crate::db::{Attr, Dat, Ent};

pub fn add(ent: impl Into<Ent>, attr: impl Into<Attr>, dat: impl Into<Dat>) -> Datom {
    Datom {
        ent: ent.into(),
        attr: attr.into(),
        dat: dat.into(),
        dir: Dir::In,
    }
}

pub fn del(ent: impl Into<Ent>, attr: impl Into<Attr>, dat: impl Into<Dat>) -> Datom {
    Datom {
        ent: ent.into(),
        attr: attr.into(),
        dat: dat.into(),
        dir: Dir::Out,
    }
}