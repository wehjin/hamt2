use crate::db::{Attr, Ein};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, Index};
use crate::db;
use crate::db::schema::attr_spec::AttrSpec;
use crate::db::schema::attribute::Attribute;
use crate::db::schema::cardinality::Cardinality;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttrTable {
    map: HashMap<Attr, Attribute>,
    by_ein: HashMap<Ein, Attr>,
}

impl AttrTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            by_ein: HashMap::new(),
        }
    }

    pub fn find_attr(&self, ein: Ein) -> Option<&Attr> {
        self.by_ein.get(&ein)
    }

    pub fn insert(&mut self, attribute: Attribute) {
        self.by_ein.insert(attribute.ein, attribute.attr().clone());
        let key = attribute.attr().clone();
        self.map.insert(key, attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = Attribute>) {
        for attribute in attributes {
            self.insert(attribute);
        }
    }

    pub fn starter() -> Self {
        let mut attr_table = Self::new();
        attr_table.extend(Self::starter_attributes());
        attr_table
    }

    fn starter_attributes() -> [Attribute; 2] {
        [
            Attribute::new(
                Ein::DB_IDENT,
                AttrSpec {
                    attr: db::ident(),
                    cardinality: Cardinality::One,
                },
            ),
            Attribute::new(
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
    type Output = Attribute;
    fn index(&self, key: Attr) -> &Self::Output {
        &self.map[&key]
    }
}

impl Deref for AttrTable {
    type Target = HashMap<Attr, Attribute>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
