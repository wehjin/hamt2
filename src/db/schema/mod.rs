use crate::db::attr_loader::AttributeLoader;
use crate::db::attr_table::AttrTable;
use crate::db::component::db_trie;
use crate::db::find::Find;
use crate::db::Db;
use crate::db::{Attr, Txid};
use crate::space::Space;
use crate::trie::SpaceTrie;
use crate::{db, LoadError, TransactError};
use attribute::Attribute;
use std::ops::{Deref, DerefMut, Index};

pub mod attr_loader;
pub mod attr_spec;
pub mod attr_table;
pub mod attribute;
pub mod cardinality;

#[derive(Debug, Clone)]
pub struct Schema {
    attr_table: AttrTable,
}

impl Schema {
    pub fn starter() -> Self {
        Self {
            attr_table: AttrTable::starter(),
        }
    }
    pub fn contains(&self, attr: &Attr) -> bool {
        self.attr_table.contains_key(attr)
    }
    pub fn insert(&mut self, attribute: Attribute) {
        self.attr_table.insert(attribute);
    }
    pub fn extend(&mut self, attributes: impl IntoIterator<Item = Attribute>) {
        self.attr_table.extend(attributes);
    }

    pub async fn save<T: Space>(
        &self,
        mut trie: SpaceTrie<T>,
        txid: Txid,
    ) -> Result<SpaceTrie<T>, TransactError> {
        for (_, attribute) in self.attr_table.iter() {
            let ein = attribute.ein;
            trie = db_trie::add(
                trie,
                &self.attr_table,
                ein,
                db::IDENT,
                attribute.ident().into(),
                &txid,
            )
            .await?;
            trie = db_trie::add(
                trie,
                &self.attr_table,
                ein,
                db::CARDINALITY,
                attribute.cardinality().into(),
                &txid,
            )
            .await?
        }
        Ok(trie)
    }
    pub async fn load<T: Space>(attrs: Vec<Attr>, db: &Db<T>) -> Result<Self, LoadError> {
        let mut schema = db.schema.clone();
        {
            // Find attributes for the requested attrs in the db.
            let loader = AttributeLoader::new(attrs.clone());
            let attributes = loader.apply(&db.trie, &db.schema).await?;
            schema.extend(attributes);
            // Confirm we have found an attribute for every requested attr.
            for attr in attrs.iter() {
                if !schema.contains(attr) {
                    return Err(LoadError::UnknownAttr(*attr));
                }
            }
        }
        Ok(schema)
    }
}

impl Index<Attr> for Schema {
    type Output = Attribute;
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

impl DerefMut for Schema {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attr_table
    }
}
