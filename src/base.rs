use serde::{Deserialize, Serialize};

pub enum Datom {
    Add(Ent, Attr, Val),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Ent(pub i64);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Attr(pub &'static str);
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Val {
    UInt(u64),
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Tx(pub u64);
impl Tx {
    pub fn next(&self) -> Tx {
        Tx(self.0 + 1)
    }
}
