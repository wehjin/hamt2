use crate::base::{Ent, Tx};
use crate::client::{QueryError, TransactError};
use iroh_blobs::api::Store;
use iroh_docs::api::Doc;
use iroh_docs::store::Query;
use iroh_docs::AuthorId;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub struct Db {
    pub max_ent: Ent,
    pub max_tx: Tx,
}

impl Db {
    pub fn new() -> Self {
        Self {
            max_ent: Ent(0),
            max_tx: Tx(0),
        }
    }
    pub async fn transact(self, author: AuthorId, doc: &Doc) -> Result<(), TransactError> {
        let db_json = serde_json::to_string(&self)?;
        doc.set_bytes(author, "db", db_json).await?;
        Ok(())
    }
    pub async fn query(doc: &Doc, store: &Store) -> Result<Self, QueryError> {
        let query = Query::key_exact("db");
        match doc.get_one(query).await? {
            Some(entry) => {
                let hash = entry.content_hash();
                let mut db_json = String::new();
                store.reader(hash).read_to_string(&mut db_json).await?;
                let db = serde_json::from_str::<Db>(&db_json)?;
                Ok(db)
            }
            None => Ok(Self::new()),
        }
    }
}
