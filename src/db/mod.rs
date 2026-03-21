use crate::hamt::space::mem::MemSpace;

pub mod txid;

pub struct Db {
    space: MemSpace,
}
impl Db {
    pub fn new() -> Self {
        let space = MemSpace::new();
        Self { space }
    }

    pub fn transact(self, datoms: impl AsRef<[Datom]>) -> Self {
        let Db { space } = self;
        let datoms = datoms.as_ref();
        for datom in datoms {
            match datom {
                Datom::Add(e, a, v) => {
                    println!("{:?} {:?} {:?}", e, a, v);
                }
            }
        }
        Self { space }
    }
}

pub enum Datom {
    Add(Ent, Attr, Val),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ent(i32);
impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self(i)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(&'static str);
impl From<&'static str> for Attr {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Val {
    U32(u32),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn db_works() {
        let db = Db::new();
    }
}
