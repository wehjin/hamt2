use crate::iroh_db::core::{Attr, Ent, Val};
use crate::iroh_db::client::db::Db;
use crate::iroh_db::client::keys::{eavt_ea_key, val_from_eavt_full_key};
use crate::iroh_db::client::values::DATOM_ADDED;
use crate::QueryError;
use iroh_blobs::api::Store;
use iroh_docs::api::Doc;
use iroh_docs::store::Query;
use tokio::io::AsyncReadExt;

pub struct Reader {
    pub doc: Doc,
    pub store: Store,
}

impl Reader {
    pub async fn query_db(&self) -> Result<Db, QueryError> {
        Db::query(&self.doc, &self.store).await
    }

    pub async fn query_value(&self, e: Ent, a: &Attr) -> Result<Option<Val>, QueryError> {
        let ea_key = eavt_ea_key(&e, &a);
        let query = Query::key_prefix(ea_key);
        let entry = self.doc.get_one(query).await?;
        if let Some(entry) = entry {
            let hash = entry.content_hash();
            let mut value = String::new();
            self.store.reader(hash).read_to_string(&mut value).await?;
            if value == DATOM_ADDED {
                let key = entry.key();
                let eavt_key = str::from_utf8(key)?;
                let val = val_from_eavt_full_key(eavt_key)?;
                Ok(Some(val))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
