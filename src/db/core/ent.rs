#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ent(pub i32);

impl Ent {
    pub const DB_IDENT: Ent = Ent(-1);

    pub fn i32(&self) -> i32 {
        self.0
    }
    pub fn to_id(&self) -> i32 {
        self.0
    }
}

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self(i)
    }
}
