use crate::reader::Reader;
use iroh::endpoint::BindError;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr};
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::BlobsProtocol;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;
use std::rc::Rc;

pub struct Client {
    endpoint: Endpoint,
    _docs: Docs,
    _router: Router,
    doc: Rc<Doc>,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),

    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

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
        Ok(Self {
            endpoint,
            _docs: docs,
            _router: router,
            doc: Rc::new(doc),
        })
    }

    pub fn to_reader(&self) -> Reader {
        Reader::from(self.doc.clone())
    }

    pub fn to_endpoint_addr(&self) -> EndpointAddr {
        self.endpoint.addr()
    }
}
