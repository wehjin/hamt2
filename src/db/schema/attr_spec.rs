use crate::db::cardinality::Cardinality;
use crate::db::Attr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttrSpec {
    pub attr: Attr,
    pub cardinality: Cardinality,
}

impl From<Attr> for AttrSpec {
    fn from(attr: Attr) -> Self {
        Self {
            attr,
            cardinality: Cardinality::One,
        }
    }
}
