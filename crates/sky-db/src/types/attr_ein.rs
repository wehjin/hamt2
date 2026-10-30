use crate::Ein;
use serde::{Deserialize, Serialize};

/// An attr-ein is an `Ein` that refers to an attribute (as opposed to a data
/// entity). Attributes are themselves entities, so an `AttrEin` and an `Ein`
/// name the same underlying id; the newtype keeps the two roles distinct at
/// the type level.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AttrEin(Ein);

impl AttrEin {
    pub fn ein(&self) -> Ein {
        self.0
    }

    pub fn to_i32(&self) -> i32 {
        self.0.to_i32()
    }
}

impl From<Ein> for AttrEin {
    fn from(ein: Ein) -> Self {
        AttrEin(ein)
    }
}

impl From<i32> for AttrEin {
    fn from(ein: i32) -> Self {
        AttrEin(Ein(ein))
    }
}
