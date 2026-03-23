use serde::{Deserialize, Serialize};
use std::fmt::Formatter;
use std::ops::Add;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Seg(pub u32);

impl std::fmt::Display for Seg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Seg").finish()
    }
}

impl Add<u32> for Seg {
    type Output = Seg;
    fn add(self, rhs: u32) -> Self::Output {
        Seg(self.0 + rhs)
    }
}
