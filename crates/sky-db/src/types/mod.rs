mod attr;
mod dat;
mod ein;
mod ent;
mod fill;
mod find_result;
mod schema;
mod val;

pub use crate::_internal::schema::*;
use serde::{Deserialize, Serialize};

mod txid;
pub use attr::*;
pub use dat::*;
pub use ein::*;
pub use ent::*;
pub use fill::*;
pub use find_result::*;
pub use schema::*;

pub use txid::*;
pub use val::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Datom {
    pub ent: Ent,
    pub attr: Attr,
    pub dat: Dat,
    pub dir: Dir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    In,
    Out,
}
