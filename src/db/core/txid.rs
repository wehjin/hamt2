use std::ops::Add;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Txid(u32);

impl From<u32> for Txid {
    fn from(u: u32) -> Self {
        Self(u)
    }
}
impl Add<u32> for Txid {
    type Output = Txid;
    fn add(self, rhs: u32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Txid {
    pub(crate) const SETUP: Txid = Txid(0);
    pub const FLOOR: Txid = Txid(1);
    pub fn u32(&self) -> u32 {
        self.0
    }
}
