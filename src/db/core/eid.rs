use std::ops::AddAssign;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Eid(pub i32);

impl Eid {
    pub const DB_IDENT: Self = Self(-1);
    pub fn to_i32(&self) -> i32 {
        self.0
    }
}

impl AddAssign<i32> for Eid {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}