use crate::db;
use crate::db::AttrName;
use crate::db::attr_spec::AttrSpec;
use crate::db::cardinality::Cardinality;
use crate::db::db_trie::AttrEin;
use crate::types::schema::attribute::Attribute;
use sky_types::db::{Attr, Ein};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Index};

#[derive(Debug, Clone)]
pub struct AttrTable {
    map: HashMap<Attr, Attribute>,
}

impl AttrTable {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn find_attr(&self, attr_ein: AttrEin) -> Option<&Attr> {
        for (attr, attribute) in self.map.iter() {
            if attr_ein.has_ein(attribute.ein) {
                return Some(attr);
            }
        }
        None
    }

    pub fn find_attr_by_name(&self, attr_name: &AttrName) -> Option<&Attr> {
        for attr in self.map.keys() {
            if attr.as_ident() == &attr_name.0 {
                return Some(attr);
            }
        }
        None
    }

    pub fn insert(&mut self, attribute: Attribute) {
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

impl DerefMut for AttrTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
