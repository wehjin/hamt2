use crate::Attr;
use crate::AttrEin;
use crate::Datom;
use crate::Ein;
use crate::attr_table::AttrTable;
use crate::cardinality::Cardinality;
use attribute::AttributeDetails;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, Index};

pub mod attr_spec;
pub mod attr_table;
pub mod attribute;
pub mod cardinality;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    pub attr_table: AttrTable,
}

impl Default for Schema {
    fn default() -> Self {
        Self::starter()
    }
}

impl Schema {
    pub fn starter() -> Self {
        Self {
            attr_table: AttrTable::starter(),
        }
    }
    pub fn get_details(&self, attr: impl Into<Attr>) -> Option<AttributeDetails> {
        self.attr_table.get(attr)
    }
    pub fn list_details(&self) -> Vec<AttributeDetails> {
        self.attr_table.list()
    }

    pub fn contains(&self, attr: &Attr) -> bool {
        self.attr_table.contains_key(attr)
    }
    pub fn insert(&mut self, attribute: AttributeDetails) {
        self.attr_table.insert(attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = AttributeDetails>) {
        self.attr_table.extend(attributes);
    }
    pub fn find_attr(&self, ein: Ein) -> Option<&Attr> {
        self.attr_table.find_attr(ein)
    }

    pub fn resolve(&self, ident: &Attr) -> Option<AttrEin> {
        self.attr_table.resolve(ident)
    }

    pub fn cardinality_of(&self, ein: AttrEin) -> Cardinality {
        self.attr_table.cardinality_of(ein)
    }

    pub fn apply(&mut self, datoms: &[Datom]) {
        self.attr_table.apply(datoms);
    }

    pub fn to_attrs(&self) -> Vec<Attr> {
        self.attr_table.to_attrs()
    }
}

impl Index<Attr> for Schema {
    type Output = AttributeDetails;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.attr_table[key]
    }
}

impl Deref for Schema {
    type Target = AttrTable;
    fn deref(&self) -> &Self::Target {
        &self.attr_table
    }
}
