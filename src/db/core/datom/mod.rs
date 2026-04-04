use attr::Attr;
use ent::Ent;
use crate::db::Dat;
use std::fmt::Debug;

pub mod dat;
pub mod ent;
pub mod attr;

#[derive(Debug, Eq, PartialEq)]
pub enum Datom {
    Add(Ent, Attr, Dat),
}
