use crate::{Ein, Val};


/// This might be better holding an Entity.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bind(pub Ein, pub Val);

impl Bind {
    pub fn new(ein: impl Into<Ein>, val: impl Into<Val>) -> Self {
        Self(ein.into(), val.into())
    }
    pub fn ein(&self) -> &Ein {
        &self.0
    }
    pub fn val(&self) -> &Val {
        &self.1
    }
}

impl From<(i32, Val)> for Bind {
    fn from(value: (i32, Val)) -> Self {
        let ein = Ein::from(value.0);
        let val = value.1;
        Self(ein, val)
    }
}
