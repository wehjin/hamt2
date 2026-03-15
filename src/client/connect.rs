use std::rc::Rc;
use iroh::endpoint::BindError;
use iroh::Endpoint;
use iroh_blobs::store::mem::MemStore;
use iroh_gossip::Gossip;
use iroh_docs::protocol::Docs;
use iroh::protocol::Router;
use iroh_blobs::BlobsProtocol;
use crate::client::{Client, Loader};
use crate::reader::Reader;

impl Client {
    pub async fn connect() -> Result<Self, ClientError> {
        let endpoint = Endpoint::builder().bind().await?;
        let blobs = MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::memory()
            .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
            .await?;
        let builder = Router::builder(endpoint.clone());
        let router = builder
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&blobs, None))
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();
        let doc = docs.create().await?;
        let loader = Rc::new(Loader {
            segment: None,
            root_start: None,
        });
        Ok(Self {
            _endpoint: endpoint,
            _docs: docs,
            _router: router,
            _doc: Rc::new(doc),
            loader,
        })
    }

    pub fn to_reader(&self) -> Reader {
        let loader = self.loader.clone();
        Reader::new(loader)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),

    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}