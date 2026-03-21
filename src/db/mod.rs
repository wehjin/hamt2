use crate::hamt::space::mem::MemSpace;
use crate::hamt::trie::space::SpaceTrie;
use crate::QueryError;
use crate::TransactError;

pub mod txid;

pub struct Db {
    space: MemSpace,
}
impl Db {
    pub fn new() -> Self {
        let space = MemSpace::new();
        Self { space }
    }

    pub fn transact(self, datoms: Vec<Datom>) -> Result<Self, TransactError> {
        let Self { space } = self;
        let mut trie = SpaceTrie::connect(&space)?;
        for datom in datoms {
            trie = match datom {
                Datom::Add(e, a, v) => transact_add(trie, e, a, v)?,
            }
        }
        Ok(Self { space })
    }
    pub fn query(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
        Ok(None)
    }
}

fn transact_add(trie: SpaceTrie, e: Ent, a: Attr, v: Val) -> Result<SpaceTrie, TransactError> {
    Ok(trie)
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
