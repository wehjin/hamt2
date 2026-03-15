use crate::base::{Attr, Ent, Val};

pub enum Reader {
    Empty,
}

impl Reader {
    pub fn query_value(&self, _entity: Ent, _attribute: Attr) -> Option<Val> {
        match self {
            Reader::Empty => None,
        }
    }
}
