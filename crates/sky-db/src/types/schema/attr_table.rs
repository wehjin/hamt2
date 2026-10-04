use crate::attr_spec::AttrSpec;
use crate::attribute::AttributeDetails;
use crate::cardinality::Cardinality;
use crate::{Attr, Ein, db};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, Index};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttrTable {
    map: HashMap<Attr, AttributeDetails>,
    by_ein: HashMap<Ein, Attr>,
}

impl AttrTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            by_ein: HashMap::new(),
        }
    }

    pub fn to_attrs(&self) -> Vec<Attr> {
        self.map.keys().cloned().collect()
    }

    pub fn find_attr(&self, ein: Ein) -> Option<&Attr> {
        self.by_ein.get(&ein)
    }

    pub fn get(&self, attr: impl Into<Attr>) -> Option<AttributeDetails> {
        self.map.get(&attr.into()).cloned()
    }

    pub fn list(&self) -> Vec<AttributeDetails> {
        self.map.values().cloned().collect()
    }

    pub fn insert(&mut self, attribute: AttributeDetails) {
        self.by_ein.insert(attribute.ein, attribute.attr().clone());
        let key = attribute.attr().clone();
        self.map.insert(key, attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = AttributeDetails>) {
        for attribute in attributes {
            self.insert(attribute);
        }
    }

    pub fn starter() -> Self {
        let mut attr_table = Self::new();
        attr_table.extend(Self::starter_attributes());
        attr_table
    }

    fn starter_attributes() -> [AttributeDetails; 2] {
        [
            AttributeDetails::new(
                Ein::DB_IDENT,
                AttrSpec {
                    attr: db::ident(),
                    cardinality: Cardinality::One,
                },
            ),
            AttributeDetails::new(
                Ein::DB_CARDINALITY,
                AttrSpec {
                    attr: db::cardinality(),
                    cardinality: Cardinality::One,
                },
            ),
        ]
    }
}

impl Index<Attr> for AttrTable {
    type Output = AttributeDetails;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.map[&key]
    }
}

impl Deref for AttrTable {
    type Target = HashMap<Attr, AttributeDetails>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
