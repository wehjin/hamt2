use crate::db::Val;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cardinality {
    One,
    Many,
}

impl Into<Val> for Cardinality {
    fn into(self) -> Val {
        match self {
            Cardinality::One => Val::U32(0),
            Cardinality::Many => Val::U32(1),
        }
    }
}

impl From<Val> for Cardinality {
    fn from(val: Val) -> Self {
        match val {
            Val::U32(0) => Cardinality::One,
            Val::U32(1) => Cardinality::Many,
            _ => panic!("Invalid cardinality: {:?}", val),
        }
    }
}
