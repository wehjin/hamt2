use iroh::protocol::Router;
use iroh::Endpoint;
use iroh_blobs::api::Store;
use iroh_blobs::store::mem::MemStore;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_docs::AuthorId;
use iroh_gossip::Gossip;

#[derive(Debug, Clone)]
pub struct IrohClient {
    _router: Router,
    pub author: AuthorId,
    pub store: Store,
    pub doc: Doc,
}

impl IrohClient {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let endpoint = Endpoint::builder().bind().await?;
        let store = MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::memory()
            .spawn(endpoint.clone(), (*store).clone(), gossip.clone())
            .await?;
        let builder = Router::builder(endpoint.clone());
        let router = builder
            .accept(
                iroh_blobs::ALPN,
                iroh_blobs::BlobsProtocol::new(&store, None),
            )
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();
        let author = docs.author_default().await?;
        let doc = docs.create().await?;
        Ok(Self {
            _router: router,
            author,
            store: (*store).clone(),
            doc,
        })
    }
}
