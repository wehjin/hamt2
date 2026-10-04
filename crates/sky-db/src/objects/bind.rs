use crate::{Ein, Val};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bind(pub Ein, pub Val);

impl Bind {
    pub fn new(ein: impl Into<Ein>, val: impl Into<Val>) -> Self {
        Self(ein.into(), val.into())
    }
}

impl From<(i32, Val)> for Bind {
    fn from(value: (i32, Val)) -> Self {
        let ein = Ein::from(value.0);
        let val = value.1;
        Self(ein, val)
    }
}
