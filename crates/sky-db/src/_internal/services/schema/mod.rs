use crate::_internal::db_trie;
use crate::trie::SkyTrieMut;
use crate::types::Txid;
use crate::{Dir, TransactError};
use crate::{Schema, db};

pub mod schema_loader;

pub async fn save(schema: &Schema, trie: &mut SkyTrieMut, txid: Txid) -> Result<(), TransactError> {
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
