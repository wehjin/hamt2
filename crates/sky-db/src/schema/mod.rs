use crate::LoadError;
use crate::db::{Db, db_trie};
use crate::find::Find;
use crate::types::Txid;
use attr_loader::AttributeLoader;
use sky_trie::Trie;
use sky_trie::storage::ReadWriteStorage;
use sky_types::db;
use sky_types::db::schema::Schema;
use sky_types::db::{Attr, Dir, TransactError};

pub mod attr_loader;

pub async fn save<S: ReadWriteStorage>(
    schema: &Schema,
    mut trie: Trie<S>,
    txid: Txid,
) -> Result<Trie<S>, TransactError> {
    for (_, attribute) in schema.attr_table.iter() {
        let ein = attribute.ein;
        trie = db_trie::with_update(
            trie,
            &schema.attr_table,
            ein,
            db::ident(),
            attribute.ident().into(),
            Dir::In,
            &txid,
        )
        .await?;
        trie = db_trie::with_update(
            trie,
            &schema.attr_table,
            ein,
            db::cardinality(),
            attribute.cardinality().into(),
            Dir::In,
            &txid,
        )
        .await?
    }
    Ok(trie)
}
pub async fn load<S: ReadWriteStorage>(
    attrs: impl AsRef<[Attr]>,
    db: &Db<S>,
) -> Result<Schema, LoadError> {
    let attrs = attrs.as_ref();
    let mut schema = db.schema.clone();
    {
        // Find attributes for the requested attrs in the db.
        let loader = AttributeLoader::new(attrs);
        let attributes = loader.apply(&db.trie, &db.schema).await;
        schema.extend(attributes);
        // Confirm we have found an attribute for every requested attr.
        for attr in attrs.iter() {
            if !schema.contains(attr) {
                return Err(LoadError::UnknownAttr(attr.clone()));
            }
        }
    }
    Ok(schema)
}
