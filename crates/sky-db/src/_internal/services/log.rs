use std::path::Path;

use futures::StreamExt;
use iroh::protocol::Router;
use iroh::{Endpoint, SecretKey, endpoint::presets};
use iroh_blobs::BlobsProtocol;
use iroh_blobs::store::fs::FsStore;
use iroh_docs::api::Doc;
use iroh_docs::api::protocol::{AddrInfoOptions, ShareMode};
use iroh_docs::engine::LiveEvent;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{AuthorId, DocTicket, NamespaceId};
use iroh_gossip::net::Gossip;

use crate::Tx;

/// The transaction log: an iroh-docs replica holding one entry per transaction.
#[derive(Debug, Clone)]
pub struct Log {
    // Held to keep the endpoint, router, and docs engine (and their background tasks) alive.
    #[allow(dead_code)]
    router: Router,
    #[allow(dead_code)]
    endpoint: Endpoint,
    blobs: FsStore,
    #[allow(dead_code)]
    docs: Docs,
    doc: Doc,
    author: AuthorId,
}

impl Log {
    async fn setup(
        secret_key: SecretKey,
        path: &Path,
    ) -> anyhow::Result<(Router, Endpoint, FsStore, Docs, AuthorId)> {
        let endpoint = Endpoint::builder(presets::Minimal)
            .secret_key(secret_key)
            .bind()
            .await?;
        let blobs = FsStore::load(path.join("blobs")).await?;
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::persistent(path.to_path_buf())
            .spawn(endpoint.clone(), blobs.clone().into(), gossip.clone())
            .await?;
        let router = Router::builder(endpoint.clone())
            .accept(iroh_blobs::ALPN, BlobsProtocol::new(&blobs, None))
            .accept(iroh_docs::ALPN, docs.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();
        let author = docs.author_default().await?;
        Ok((router, endpoint, blobs, docs, author))
    }

    /// Creates a new log (a fresh doc).
    pub async fn create(secret_key: SecretKey, path: &Path) -> anyhow::Result<Self> {
        let (router, endpoint, blobs, docs, author) = Self::setup(secret_key, path).await?;
        let doc = docs.create().await?;
        Ok(Self {
            router,
            endpoint,
            blobs,
            docs,
            doc,
            author,
        })
    }

    /// Opens an existing log by namespace.
    pub async fn open(
        secret_key: SecretKey,
        path: &Path,
        namespace: NamespaceId,
    ) -> anyhow::Result<Self> {
        let (router, endpoint, blobs, docs, author) = Self::setup(secret_key, path).await?;
        let doc = docs
            .open(namespace)
            .await?
            .ok_or_else(|| anyhow::anyhow!("doc not found"))?;
        Ok(Self {
            router,
            endpoint,
            blobs,
            docs,
            doc,
            author,
        })
    }

    /// Imports a log from a ticket, waiting for the initial sync to finish.
    pub async fn import(
        secret_key: SecretKey,
        path: &Path,
        ticket: DocTicket,
    ) -> anyhow::Result<Self> {
        let (router, endpoint, blobs, docs, author) = Self::setup(secret_key, path).await?;
        let (doc, mut events) = docs.import_and_subscribe(ticket).await?;
        while let Some(event) = events.next().await {
            if matches!(event?, LiveEvent::PendingContentReady) {
                break;
            }
        }
        Ok(Self {
            router,
            endpoint,
            blobs,
            docs,
            doc,
            author,
        })
    }

    /// Appends a transaction to the log (key = content hash, value = serialized tx).
    pub async fn append(&self, tx: &Tx) -> anyhow::Result<()> {
        let bytes = postcard::to_stdvec(tx)?;
        self.doc
            .set_bytes(self.author, tx.hash().to_vec(), bytes)
            .await?;
        Ok(())
    }

    /// Reads all transactions currently in the log.
    pub async fn entries(&self) -> anyhow::Result<Vec<Tx>> {
        let mut txs = Vec::new();
        let stream = self.doc.get_many(Query::all()).await?;
        futures::pin_mut!(stream);
        while let Some(entry) = stream.next().await {
            let entry = entry?;
            let bytes = self.blobs.blobs().get_bytes(entry.content_hash()).await?;
            let tx: Tx = postcard::from_bytes(&bytes)?;
            txs.push(tx);
        }
        Ok(txs)
    }

    /// Recreates a fresh ticket for this log (with the current endpoint address).
    pub async fn ticket(&self) -> anyhow::Result<DocTicket> {
        self.doc
            .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
            .await
    }

    /// The namespace id of this log.
    pub fn namespace(&self) -> NamespaceId {
        self.doc.id()
    }

    /// The author id of this peer.
    pub fn author(&self) -> AuthorId {
        self.author
    }
}
