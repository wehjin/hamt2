mod attr;
mod dat;
mod ein;
mod ent;
mod find_result;
mod val;

pub use attr::*;
pub use dat::*;
pub use ein::*;
pub use ent::*;
pub use find_result::*;
pub use val::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Datom {
    pub ent: Ent,
    pub attr: Attr,
    pub dat: Dat,
    pub dir: Dir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    In,
    Out,
}
