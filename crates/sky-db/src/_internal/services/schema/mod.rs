use crate::_internal::db_trie;
use crate::traits::Find;
use crate::types::Txid;
use crate::{Db, Schema, db};
use crate::{Dir, TransactError};
use schema_loader::SchemaLoader;
use crate::trie_storage::MemEdit;

pub mod schema_loader;

pub async fn save(schema: &Schema, trie: &mut MemEdit, txid: Txid) -> Result<(), TransactError> {
    for (_, attribute) in schema.attr_table.iter() {
        let ein = attribute.ein;
        db_trie::with_update(
            trie,
            &schema.attr_table,
            ein,
            db::ident(),
            attribute.ident().into(),
            Dir::In,
            &txid,
        )
        .await?;
        db_trie::with_update(
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
    Ok(())
}
pub async fn load(db: &Db) -> Schema {
    let mut schema = db.schema.clone();
    let loader = SchemaLoader;
    let attributes = loader.apply(&db.trie, &db.schema).await;
    schema.extend(attributes);
    schema
}
