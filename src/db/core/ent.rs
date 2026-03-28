#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Ent {
    Id(i32),
}

impl Ent {
    pub const DB_IDENT: Ent = Ent::Id(-1);

    pub fn to_id(&self) -> i32 {
        match self {
            Ent::Id(id) => *id,
        }
    }
}

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self::Id(i)
    }
}
