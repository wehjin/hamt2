use crate::db::attr_spec::AttrSpec;
use crate::db::cardinality::Cardinality;
use crate::db::{Attr, Ein};

#[derive(Debug, Clone)]
pub struct Attribute {
    pub ein: Ein,
    pub spec: AttrSpec,
}

impl Attribute {
    pub const fn new(ein: Ein, spec: AttrSpec) -> Self {
        Self { ein, spec }
    }
    pub fn ein(&self) -> Ein {
        self.ein
    }
    pub fn attr(&self) -> Attr {
        self.spec.attr
    }
    pub fn ident(&self) -> &'static str {
        self.attr().as_ident()
    }
    pub fn cardinality(&self) -> Cardinality {
        self.spec.cardinality
    }
}
