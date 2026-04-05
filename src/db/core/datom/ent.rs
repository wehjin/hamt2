use crate::db::{Ein, Val};

pub fn ent(ent: impl Into<Ent>) -> Ent {
    ent.into()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Ent {
    Id(Ein),
    Temp(&'static str),
}

impl Ent {
    pub fn to_eid(&self) -> Ein {
        match self {
            Ent::Id(eid) => *eid,
            Ent::Temp(_) => panic!("Cannot directly convert temporary entity to Eid"),
        }
    }
}

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self::Id(Ein(i))
    }
}

impl From<&'static str> for Ent {
    fn from(s: &'static str) -> Self {
        Self::Temp(s)
    }
}

impl From<Val> for Ent {
    fn from(val: Val) -> Self {
        let eid = Ein::from(val);
        Ent::Id(eid)
    }
}
