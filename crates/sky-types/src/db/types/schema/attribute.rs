use crate::db::schema::attr_spec::AttrSpec;
use crate::db::schema::cardinality::Cardinality;
use crate::db::{Attr, Ein};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    pub ein: Ein,
    pub spec: AttrSpec,
}

impl Attribute {
    pub fn new(ein: Ein, spec: AttrSpec) -> Self {
        Self { ein, spec }
    }
    pub fn ein(&self) -> Ein {
        self.ein
    }
    pub fn attr(&self) -> Attr {
        self.spec.attr.clone()
    }
    pub fn ident(&self) -> &str {
        self.spec.attr.as_ident()
    }
    pub fn cardinality(&self) -> Cardinality {
        self.spec.cardinality
    }
}
