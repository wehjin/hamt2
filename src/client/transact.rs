use crate::base::{Datom, Ent, Tx};
use crate::client::values::DATOM_ADDED;
use crate::client::{keys, QueryError, TransactError};
use iroh_blobs::store::mem::MemStore;
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
    pub async fn query(doc: &Doc, store: &MemStore) -> Result<Self, QueryError> {
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

pub struct Transact<'a> {
    doc: &'a Doc,
    author: AuthorId,
    tx: Tx,
    max_ent: Ent,
}
impl<'a> Transact<'a> {
    pub async fn new(
        doc: &'a Doc,
        author: AuthorId,
        store: &MemStore,
    ) -> Result<Self, TransactError> {
        let db = Db::query(doc, store).await?;
        Ok(Self {
            doc,
            author,
            tx: db.max_tx.next(),
            max_ent: db.max_ent,
        })
    }
    pub async fn process_datum(&mut self, datom: &Datom) -> Result<(), TransactError> {
        match datom {
            Datom::Add(e, a, v) => {
                self.max_ent = self.max_ent.max(*e);
                self.doc
                    .set_bytes(self.author, keys::eavt_full_key(e, a, v, &self.tx), DATOM_ADDED)
                    .await?;
                self.doc
                    .set_bytes(self.author, keys::aevt_key(a, e, v, &self.tx), DATOM_ADDED)
                    .await?;
                self.doc
                    .set_bytes(self.author, keys::avet_key(a, v, e, &self.tx), DATOM_ADDED)
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn close(self) -> Result<(), TransactError> {
        let db = Db {
            max_ent: self.max_ent,
            max_tx: self.tx,
        };
        db.transact(self.author, &self.doc).await
    }
}
