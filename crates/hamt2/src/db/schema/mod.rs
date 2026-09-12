use crate::db::attr_loader::AttributeLoader;
use crate::db::attr_table::AttrTable;
use crate::db::component::db_trie;
use crate::db::component::db_trie::AttrEin;
use crate::db::find::Find;
use crate::db::{Attr, Db, Dir, Txid};
use crate::trie::Trie;
use crate::trie::base_storage::BaseStorageReadWrite;
use crate::{LoadError, TransactError, db};
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
    pub fn find_attr(&self, attr_ein: AttrEin) -> Option<&Attr> {
        self.attr_table.find_attr(attr_ein)
    }

    pub async fn save<S: BaseStorageReadWrite>(
        &self,
        mut trie: Trie<S>,
        txid: Txid,
    ) -> Result<Trie<S>, TransactError> {
        for (_, attribute) in self.attr_table.iter() {
            let ein = attribute.ein;
            trie = db_trie::with_update(
                trie,
                &self.attr_table,
                ein,
                db::IDENT,
                attribute.ident().into(),
                Dir::In,
                &txid,
            )
            .await?;
            trie = db_trie::with_update(
                trie,
                &self.attr_table,
                ein,
                db::CARDINALITY,
                attribute.cardinality().into(),
                Dir::In,
                &txid,
            )
            .await?
        }
        Ok(trie)
    }
    pub async fn load<S: BaseStorageReadWrite>(
        attrs: impl AsRef<[Attr]>,
        db: &Db<S>,
    ) -> Result<Self, LoadError> {
        let attrs = attrs.as_ref();
        let mut schema = db.schema.clone();
        {
            // Find attributes for the requested attrs in the db.
            let loader = AttributeLoader::new(attrs);
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
