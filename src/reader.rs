use crate::hamt::base::{Attr, Ent, Value};

pub enum Reader {
    Empty,
}

impl Reader {
    pub fn new() -> Self {
        Reader::Empty
    }
    pub fn query_value(&self, _entity: Ent, _attribute: Attr) -> Option<Value> {
        match self {
            Reader::Empty => None,
        }
    }
}
