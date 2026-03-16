use crate::base::Datom;
use crate::client::transact::Transact;
use crate::reader::Reader;
use iroh::endpoint::BindError;
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
    _endpoint: Endpoint,
    _router: Router,
    _docs: Docs,
    author: AuthorId,
    store: Store,
    doc: Doc,
}

impl Client {
    pub async fn connect() -> Result<Self, ConnectError> {
        let endpoint = Endpoint::builder().bind().await?;
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
            _endpoint: endpoint,
            _docs: docs,
            _router: router,
            author,
            store: (*store).clone(),
            doc,
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

    #[error("SpaceReadError: {0}")]
    SpaceReadError(#[from] crate::hamt::space::ReadError),

    #[error("NotAValue: {0}")]
    NotAValue(u32),

    #[error("InvalidSlotType")]
    InvalidSlotType,

    #[error("MismatchedKeys: {0} != {1}")]
    MismatchedKeys(i32, i32),

    #[error("BaseIndexOutOfBounds: {0}")]
    BaseIndexOutOfBounds(usize),
}

#[derive(thiserror::Error, Debug)]
pub enum TransactError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("SerdeJson: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Query: {0}")]
    Query(#[from] QueryError),

    #[error("HighBitInValue: {0}")]
    HighBitInValue(u32),
}

#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Endpoint bind error: {0}")]
    EndpointBindError(#[from] BindError),
}
