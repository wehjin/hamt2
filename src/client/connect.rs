use crate::client::{Client, ClientError};
use crate::reader::Reader;
use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::BlobsProtocol;
use iroh_docs::protocol::Docs;
use iroh_gossip::Gossip;
use std::rc::Rc;

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
            _endpoint: endpoint,
            _docs: docs,
            _router: router,
            _doc: Rc::new(doc),
        })
    }

    pub fn to_reader(&self) -> Reader {
        Reader::new()
    }
}

