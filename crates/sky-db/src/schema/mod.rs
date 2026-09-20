use crate::db::{Db, db_trie};
use crate::traits::Find;
use crate::types::Txid;
use schema_loader::SchemaLoader;
use sky_trie::Trie;
use sky_types::db;
use sky_types::db::schema::Schema;
use sky_types::db::{Dir, TransactError};
use sky_types::storage::ReadWriteStorage;

pub mod schema_loader;

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
pub async fn load<S: ReadWriteStorage>(db: &Db<S>) -> Schema {
    let mut schema = db.schema.clone();
    let loader = SchemaLoader;
    let attributes = loader.apply(&db.trie, &db.schema).await;
    schema.extend(attributes);
    schema
}
