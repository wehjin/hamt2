use crate::{Ein, services};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Val {
    U32(u32),
    U64(u64),
    String(String),
    Bytes(Vec<u8>),
    Ein(Ein),
}

impl Val {
    pub fn i32(&self) -> i32 {
        match self {
            Val::U32(v) => *v as i32,
            _ => panic!("Not a i32"),
        }
    }
    pub fn u32(&self) -> u32 {
        match self {
            Val::U32(v) => *v,
            _ => panic!("Not a u32"),
        }
    }
    pub fn try_into_u32(self) -> Option<u32> {
        match self {
            Val::U32(v) => Some(v),
            _ => None,
        }
    }
    pub fn u64(&self) -> u64 {
        match self {
            Val::U64(v) => *v,
            _ => panic!("Not a u64"),
        }
    }
    pub fn try_into_u64(self) -> Option<u64> {
        match self {
            Val::U64(v) => Some(v),
            _ => None,
        }
    }
    pub fn ein(&self) -> Ein {
        match self {
            Val::Ein(e) => *e,
            _ => panic!("Not an ein"),
        }
    }
    pub fn try_into_ein(self) -> Option<Ein> {
        match self {
            Val::Ein(e) => Some(e),
            _ => None,
        }
    }
}

impl Val {
    pub fn from_str(s: &str) -> Self {
        Val::String(s.to_string())
    }
    pub fn as_str(&self) -> &str {
        match self {
            Val::String(s) => s,
            _ => panic!("Not a string"),
        }
    }
    pub fn try_into_string(self) -> Option<String> {
        match self {
            Val::String(s) => Some(s),
            _ => None,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        match self {
            Val::Bytes(b) => b,
            _ => panic!("Not bytes"),
        }
    }
    pub fn try_into_bytes(self) -> Option<Vec<u8>> {
        match self {
            Val::Bytes(b) => Some(b),
            _ => None,
        }
    }
}

impl From<Ein> for Val {
    fn from(value: Ein) -> Self {
        services::val(value.to_i32())
    }
}

impl From<i32> for Val {
    fn from(i: i32) -> Self {
        Self::U32(i as u32)
    }
}

impl From<u32> for Val {
    fn from(u: u32) -> Self {
        Self::U32(u)
    }
}

impl From<u64> for Val {
    fn from(u: u64) -> Self {
        Self::U64(u)
    }
}

impl From<&str> for Val {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Val {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<Vec<u8>> for Val {
    fn from(b: Vec<u8>) -> Self {
        Self::Bytes(b)
    }
}

impl From<&[u8]> for Val {
    fn from(b: &[u8]) -> Self {
        Self::Bytes(b.to_vec())
    }
}
