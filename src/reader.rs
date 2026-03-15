use crate::base::{Attribute, Entity, Value};

pub enum Reader {
    Empty,
}

impl Reader {
    pub fn new() -> Self {
        Reader::Empty
    }
    pub fn query_value(&self, _entity: Entity, _attribute: Attribute) -> Option<Value> {
        match self {
            Reader::Empty => None,
        }
    }
}
