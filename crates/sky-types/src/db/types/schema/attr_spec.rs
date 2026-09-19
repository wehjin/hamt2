use crate::db::Attr;
use crate::db::schema::cardinality::Cardinality;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl From<&Attr> for AttrSpec {
    fn from(attr: &Attr) -> Self {
        Self {
            attr: attr.clone(),
            cardinality: Cardinality::One,
        }
    }
}

pub struct DbSpec {
    attrs_specs: Vec<AttrSpec>,
}
impl AsRef<[AttrSpec]> for DbSpec {
    fn as_ref(&self) -> &[AttrSpec] {
        &self.attrs_specs
    }
}

impl<const N: usize> From<[AttrSpec; N]> for DbSpec {
    fn from(value: [AttrSpec; N]) -> Self {
        Self {
            attrs_specs: value.to_vec(),
        }
    }
}

impl<const N: usize> From<[Attr; N]> for DbSpec {
    fn from(value: [Attr; N]) -> Self {
        Self {
            attrs_specs: value.iter().map(AttrSpec::from).collect(),
        }
    }
}

impl From<Vec<AttrSpec>> for DbSpec {
    fn from(value: Vec<AttrSpec>) -> Self {
        Self { attrs_specs: value }
    }
}

impl From<Vec<Attr>> for DbSpec {
    fn from(attrs: Vec<Attr>) -> Self {
        Self {
            attrs_specs: attrs.into_iter().map(AttrSpec::from).collect(),
        }
    }
}

impl<const N: usize> From<[&str; N]> for DbSpec {
    fn from(value: [&str; N]) -> Self {
        Self {
            attrs_specs: value
                .iter()
                .map(|s| AttrSpec::from(Attr::from(*s)))
                .collect(),
        }
    }
}

impl From<Vec<&str>> for DbSpec {
    fn from(value: Vec<&str>) -> Self {
        Self {
            attrs_specs: value
                .iter()
                .map(|s| AttrSpec::from(Attr::from(*s)))
                .collect(),
        }
    }
}
