use crate::db::{Ein, Val};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Ent {
    Id(Ein),
    Temp(String),
}

impl Ent {
    pub fn to_eid(&self) -> Ein {
        match self {
            Ent::Id(eid) => *eid,
            Ent::Temp(_) => panic!("Cannot directly convert temporary entity to Eid"),
        }
    }
}

impl From<Ein> for Ent {
    fn from(eid: Ein) -> Self {
        Ent::Id(eid)
    }
}

impl From<Val> for Ent {
    fn from(val: Val) -> Self {
        let eid = Ein::from(val);
        Ent::Id(eid)
    }
}

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self::Id(Ein(i))
    }
}

impl From<&str> for Ent {
    fn from(s: &str) -> Self {
        Self::Temp(s.to_string())
    }
}

impl From<String> for Ent {
    fn from(s: String) -> Self {
        Self::Temp(s)
    }
}
