use crate::base::{Attr, Datom, Ent, Tx, Val};
use crate::client::db::Db;
use crate::client::values::DATOM_ADDED;
use crate::client::{keys, TransactError};
use iroh_blobs::api::Store;
use iroh_docs::api::Doc;
use iroh_docs::AuthorId;

pub struct Transact<'a> {
    doc: &'a Doc,
    author: AuthorId,
    tx: Tx,
    max_ent: Ent,
}
impl<'a> Transact<'a> {
    pub async fn new(doc: &'a Doc, author: AuthorId, store: &Store) -> Result<Self, TransactError> {
        let db = Db::query(doc, store).await?;
        Ok(Self {
            doc,
            author,
            tx: db.max_tx.next(),
            max_ent: db.max_ent,
        })
    }

    pub async fn process_datum(&mut self, datom: Datom) -> Result<(), TransactError> {
        match datom {
            Datom::Add(e, a, v) => {
                self.process_add(&e, &a, &v).await?;
            }
            Datom::Id(e, av_list) => {
                for (a, v) in av_list {
                    self.process_add(&e, &a, &v).await?;
                }
            }
        }
        Ok(())
    }
    async fn process_add(&mut self, e: &Ent, a: &Attr, v: &Val) -> Result<(), TransactError> {
        self.max_ent = self.max_ent.max(*e);
        self.doc
            .set_bytes(
                self.author,
                keys::eavt_full_key(&e, &a, &v, &self.tx),
                DATOM_ADDED,
            )
            .await?;
        self.doc
            .set_bytes(
                self.author,
                keys::aevt_key(&a, &e, &v, &self.tx),
                DATOM_ADDED,
            )
            .await?;
        self.doc
            .set_bytes(
                self.author,
                keys::avet_key(&a, &v, &e, &self.tx),
                DATOM_ADDED,
            )
            .await?;
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
