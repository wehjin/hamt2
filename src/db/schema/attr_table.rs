use crate::db;
use crate::db::attr_spec::AttrSpec;
use crate::db::cardinality::Cardinality;
use crate::db::schema::attribute::Attribute;
use crate::db::{Attr, Ein};
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

    pub fn insert(&mut self, attribute: Attribute) {
        self.map.insert(attribute.attr(), attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = Attribute>) {
        for attribute in attributes {
            self.insert(attribute);
        }
    }

    pub fn starter() -> Self {
        let mut attr_table = Self::new();
        attr_table.extend(Self::STARTER_ATTRIBUTES);
        attr_table
    }

    const STARTER_ATTRIBUTES: [Attribute; 2] = [
        Attribute::new(
            Ein::DB_IDENT,
            AttrSpec {
                attr: db::IDENT,
                cardinality: Cardinality::One,
            },
        ),
        Attribute::new(
            Ein::DB_CARDINALITY,
            AttrSpec {
                attr: db::CARDINALITY,
                cardinality: Cardinality::One,
            },
        ),
    ];
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
