use crate::space::seg::Seg;
use crate::space::table::TablePos;
use crate::space::value::Val;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ValueAddr(pub Seg, pub Val);

impl fmt::Display for ValueAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValueAddr")
            .field("seg", &self.0)
            .field("val", &self.1)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TableAddr(pub Seg, pub TablePos);

impl fmt::Display for TableAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TableAddr")
            .field("seg", &self.0)
            .field("pos", &self.1)
            .finish()
    }
}
