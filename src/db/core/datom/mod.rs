use crate::db::Dat;
use attr::Attr;
use ent::Ent;
use std::fmt::Debug;

pub mod attr;
pub mod dat;
pub mod ent;

pub fn datom(ent: impl Into<Ent>, attr: impl Into<Attr>, dat: impl Into<Dat>) -> Datom {
    Datom::Add(ent.into(), attr.into(), dat.into())
}

#[derive(Debug, Eq, PartialEq)]
pub enum Datom {
    Add(Ent, Attr, Dat),
}
