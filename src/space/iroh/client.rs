use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::{Endpoint, SecretKey};
use iroh_blobs::api::Store;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::store::mem::MemStore;
use iroh_docs::api::Doc;
use iroh_docs::protocol::{Builder, Docs};
use iroh_docs::{AuthorId, NamespaceId};
use iroh_gossip::Gossip;
use std::path::Path;

#[cfg(test)]
mod tests {
    use crate::space::iroh::client::IrohClient;
    use iroh::SecretKey;
    use iroh_docs::store::Query;

    #[tokio::test]
    async fn file_client_works() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
        let mut doc_id = None;
        {
            let client = IrohClient::connect(temp_dir.path(), doc_id, secret_key.clone()).await?;
            doc_id = Some(client.doc.id());
            client.doc.set_bytes(client.author, "key", "value").await?;
            client.router.shutdown().await?;
        }
        {
            let client = IrohClient::connect(temp_dir.path(), doc_id, secret_key).await?;
            let result = client.doc.get_one(Query::key_exact("key")).await?;
            let entry = result.expect("entry should exist");
            assert_eq!("key".as_bytes(), entry.key());
            client.router.shutdown().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IrohClient {
    pub author: AuthorId,
    pub store: Store,
    pub doc: Doc,
    pub router: Router,
}

impl IrohClient {
    pub async fn new_mem() -> anyhow::Result<Self> {
        let secret_key = SecretKey::from_bytes(&[0x01u8; 32]);
        let store = MemStore::new();
        let docs_builder = Docs::memory();
        Self::build((*store).clone(), docs_builder, None, secret_key).await
    }

    pub async fn connect(
        path: impl AsRef<Path>,
        doc_id: Option<NamespaceId>,
        secret_key: SecretKey,
    ) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let blobs_path = path.join("blobs");
        let blobs_store = FsStore::load(blobs_path).await?;
        let docs_path = path.join("docs");
        tokio::fs::create_dir_all(&docs_path).await?;
        let docs_store = Docs::persistent(docs_path);
        Self::build((*blobs_store).clone(), docs_store, doc_id, secret_key).await
    }

    async fn build(
        blobs_store: Store,
        docs_builder: Builder,
        doc_id: Option<NamespaceId>,
        secret_key: SecretKey,
    ) -> anyhow::Result<Self> {
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(secret_key)
            .bind()
            .await?;
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = docs_builder
            .spawn(endpoint.clone(), blobs_store.clone(), gossip.clone())
            .await?;
        let router = Router::builder(endpoint.clone())
            .accept(
                iroh_blobs::ALPN,
                iroh_blobs::BlobsProtocol::new(&blobs_store, None),
            )
            .accept(iroh_gossip::ALPN, gossip)
            .accept(iroh_docs::ALPN, docs.clone())
            .spawn();
        let author = docs.author_default().await?;
        let doc = if let Some(id) = doc_id {
            docs.open(id).await?.expect("doc not found")
        } else {
            docs.create().await?
        };
        Ok(Self {
            author,
            store: blobs_store,
            doc,
            router,
        })
    }
}
