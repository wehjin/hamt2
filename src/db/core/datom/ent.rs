use crate::db::Eid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Ent {
    Id(Eid),
    Temp(&'static str),
}

impl Ent {
    pub fn to_eid(&self) -> Eid {
        match self {
            Ent::Id(eid) => *eid,
            Ent::Temp(_) => panic!("Cannot directly convert temporary entity to Eid"),
        }
    }
}

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self::Id(Eid(i))
    }
}

impl From<&'static str> for Ent {
    fn from(s: &'static str) -> Self {
        Self::Temp(s)
    }
}
