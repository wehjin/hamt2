use std::hash::{DefaultHasher, Hash, Hasher};

pub enum Datom {
    Add(Ent, Attr, Val),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Ent(pub i32);

impl From<i32> for Ent {
    fn from(i: i32) -> Self {
        Self(i)
    }
}

impl Ent {
    pub fn i32(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(&'static str, Ent);

impl From<&'static str> for Attr {
    fn from(s: &'static str) -> Self {
        let ent = {
            let mut hasher = DefaultHasher::new();
            s.hash(&mut hasher);
            let hash = hasher.finish() as i32;
            Ent(hash)
        };
        Self(s, ent)
    }
}

impl Attr {
    pub fn to_ent(&self) -> Ent {
        self.1
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Val {
    U32(u32),
}

impl From<u32> for Val {
    fn from(u: u32) -> Self {
        Self::U32(u)
    }
}

impl Val {
    pub fn u32(&self) -> u32 {
        match self {
            Val::U32(v) => *v,
        }
    }
}