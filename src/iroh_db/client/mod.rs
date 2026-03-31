use crate::iroh_db::client::transact::Transact;
use crate::iroh_db::core::Datom;
use crate::iroh_db::reader::Reader;
use crate::TransactError;
use iroh::endpoint::{presets, BindError};
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::api::Store;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::BlobsProtocol;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_docs::AuthorId;
use iroh_gossip::Gossip;

pub mod db;
pub mod keys;
pub mod transact;
pub mod values;

pub struct Client {
    pub author: AuthorId,
    store: Store,
    pub doc: Doc,
    pub router: Router,
}

impl Client {
    pub async fn connect() -> Result<Self, ConnectError> {
        let endpoint = Endpoint::bind(presets::N0).await?;
        let store = MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::memory()
            .spawn(endpoint.clone(), (*store).clone(), gossip.clone())
            .await?;
        let builder = Router::builder(endpoint.clone());
        let router = builder
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&store, None))
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();
        let author = docs.author_default().await?;
        let doc = docs.create().await?;
        Ok(Self {
            author,
            store: (*store).clone(),
            doc,
            router,
        })
    }

    pub async fn transact(&mut self, datoms: Vec<Datom>) -> Result<(), TransactError> {
        let mut transact = Transact::new(&self.doc, self.author, &self.store).await?;
        for datom in datoms {
            transact.process_datum(datom).await?;
        }
        transact.close().await?;
        Ok(())
    }

    pub fn to_reader(&self) -> Reader {
        Reader {
            doc: self.doc.clone(),
            store: self.store.clone(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),
}
