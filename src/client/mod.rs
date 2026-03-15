use crate::base::{Attr, Datom, Ent, Val};
use crate::client::keys::{eavt_ea_key, val_from_eavt_full_key};
use crate::client::transact::Transact;
use crate::client::values::DATOM_ADDED;
use iroh::endpoint::BindError;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::store::mem::MemStore;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::AuthorId;
use std::rc::Rc;
use tokio::io::AsyncReadExt;

pub mod connect;
pub mod keys;
pub mod values;

pub mod transact;

pub struct Client {
    _endpoint: Endpoint,
    _router: Router,
    _docs: Docs,
    author: AuthorId,
    store: Rc<MemStore>,
    doc: Rc<Doc>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Query: {0}")]
    Query(#[from] QueryError),
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Io: {0}")]
    Io(#[from] std::io::Error),

    #[error("Utf8: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Key: {0}")]
    Key(#[from] keys::Error),
}

impl Client {
    pub async fn transact(&mut self, datoms: &[Datom]) -> Result<(), TransactError> {
        let mut transact = Transact::new(&self.doc, self.author, &self.store).await?;
        for datom in datoms {
            transact.process_datum(datom).await?;
        }
        transact.close().await?;
        Ok(())
    }

    pub async fn query_value(&self, e: Ent, a: Attr) -> Result<Option<Val>, QueryError> {
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
