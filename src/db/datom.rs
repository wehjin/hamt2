use crate::trie::mem::value::MemValue;
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(Debug, Eq, PartialEq)]
pub enum Datom {
    Add(Ent, Attr, Val),
}

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(pub &'static str, pub &'static str);

impl Attr {
    pub const DB_IDENT: Attr = Attr("db", "ident");
    pub fn to_ident(&self) -> String {
        format!("{}/{}", self.0, self.1)
    }
}

impl Display for Attr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_ident().as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Val {
    U32(u32),
    String(String),
}

impl Val {
    pub fn u32(&self) -> u32 {
        match self {
            Val::U32(v) => *v,
            Val::String(_) => panic!("Not a u32"),
        }
    }
    pub fn try_into_u32(self) -> Option<u32> {
        match self {
            Val::U32(v) => Some(v),
            Val::String(_) => None,
        }
    }
}

impl Val {
    pub fn from_str(s: &str) -> Self {
        Val::String(s.to_string())
    }
    pub fn as_str(&self) -> &str {
        match self {
            Val::U32(_) => panic!("Not a string"),
            Val::String(s) => s,
        }
    }
    pub fn try_into_string(self) -> Option<String> {
        match self {
            Val::U32(_) => None,
            Val::String(s) => Some(s.clone()),
        }
    }
}

impl From<MemValue> for Val {
    fn from(value: MemValue) -> Self {
        match value {
            MemValue::U32(v) => Val::U32(v),
            MemValue::MapBase(_) => unreachable!(),
        }
    }
}

impl From<u32> for Val {
    fn from(u: u32) -> Self {
        Self::U32(u)
    }
}

impl From<&str> for Val {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}
