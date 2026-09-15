use crate::db::Val;
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Ein(pub i32);

impl Ein {
    pub const DB_IDENT: Self = Self(0);
    pub const DB_CARDINALITY: Self = Self(1);
    pub const DB_MAX: Self = Self(2);

    pub fn to_i32(&self) -> i32 {
        debug_assert!(self.0 >= 0);
        self.0
    }
}

impl AddAssign<i32> for Ein {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}

impl From<Val> for Ein {
    fn from(val: Val) -> Self {
        let ein = val.u32() as i32;
        debug_assert!(ein >= 0);
        Ein(ein)
    }
}

impl From<i32> for Ein {
    fn from(i: i32) -> Self {
        debug_assert!(i >= 0);
        Ein(i)
    }
}
