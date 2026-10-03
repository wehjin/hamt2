use crate::attr_spec::AttrSpec;
use crate::cardinality::Cardinality;
use crate::{Attr, Ein};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AttributeDetails {
    pub ein: Ein,
    pub spec: AttrSpec,
}

impl AttributeDetails {
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
