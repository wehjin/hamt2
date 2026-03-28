use crate::trie::mem::value::MemValue;

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
