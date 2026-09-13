use crate::db::types::attr::Attr;
use crate::db::{Dat, Dir};
use ent::Ent;
use std::fmt::Debug;

pub mod dat;
pub mod ent;

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

#[derive(Debug, PartialEq, Eq)]
pub struct Datom {
    pub ent: Ent,
    pub attr: Attr,
    pub dat: Dat,
    pub dir: Dir,
}
