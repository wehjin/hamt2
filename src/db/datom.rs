use crate::hamt::trie::mem::value::MemValue;
use std::hash::Hash;

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
    pub fn to_id(&self) -> i32 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Attr(pub &'static str);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Val {
    U32(u32),
    String(String),
}

impl From<MemValue> for Val {
    fn from(value: MemValue) -> Self {
        match value {
            MemValue::U32(v) => Val::U32(v),
            MemValue::String(v) => Val::String(v),
            MemValue::MapBase(_) => unreachable!(),
        }
    }
}

impl From<u32> for Val {
    fn from(u: u32) -> Self {
        Self::U32(u)
    }
}

impl From<&Attr> for Val {
    fn from(a: &Attr) -> Self {
        Self::String(a.0.to_string())
    }
}

impl Val {
    pub fn u32(&self) -> u32 {
        match self {
            Val::U32(v) => *v,
            Val::String(_) => panic!("Not a u32"),
        }
    }
}
