use crate::db::core::attr::Attr;
use crate::db::core::ent::Ent;
use crate::db::Val;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq)]
pub enum Datom {
    Add(Ent, Attr, Val),
}
