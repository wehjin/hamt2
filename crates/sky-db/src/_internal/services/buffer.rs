use std::path::Path;
use thiserror::Error;

use iroh::{Endpoint, SecretKey, endpoint::presets, protocol::ProtocolHandler};
use iroh_blobs::store::fs::FsStore;
use iroh_docs::NamespaceId;
use iroh_docs::protocol::Docs;
use iroh_gossip::net::Gossip;

use crate::trie::MapBase;

/// The doc key that points at the current root MapBase.
pub const ROOT_KEY: &[u8] = b"root";

/// The named tag that pins the root blob against garbage collection.
pub const ROOT_TAG: &[u8] = b"root-blob";

/// The filename used to store the doc's namespace id.
pub const NAMESPACE_FILE: &str = "namespace";

#[derive(Error, Debug)]
pub enum BufferInitError {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

/// Initializes the iroh-backed state at `path`.
///
/// Spins up the iroh endpoint, blob store, gossip, and docs; writes an empty
/// root [`MapBase`] as a blob (pinned with a named tag); and records it in a
/// doc entry. Returns the doc's [`NamespaceId`].
pub async fn init(path: &Path, secret_key: &SecretKey) -> Result<NamespaceId, BufferInitError> {
    Ok(init_inner(path, secret_key).await?)
}

async fn init_inner(path: &Path, secret_key: &SecretKey) -> anyhow::Result<NamespaceId> {
    let endpoint = Endpoint::builder(presets::Minimal)
        .secret_key(secret_key.clone())
        .bind()
        .await?;

    let blobs = FsStore::load(path.join("blobs")).await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let docs = Docs::persistent(path.to_path_buf())
        .spawn(endpoint.clone(), blobs.clone().into(), gossip)
        .await?;

    // Empty root -> blob -> hash, pinned with a named tag.
    let root = MapBase::empty();
    let bytes = postcard::to_stdvec(&root)?;
    let haf = blobs.add_bytes(bytes.clone()).with_named_tag(ROOT_TAG).await?;
    let hash = haf.hash;

    // Record the root in the doc.
    let author = docs.author_default().await?;
    let doc = docs.create().await?;
    doc.set_hash(author, ROOT_KEY, hash, bytes.len() as u64).await?;
    let namespace = doc.id();

    // Persist the namespace id so `load` can find the doc.
    std::fs::write(path.join(NAMESPACE_FILE), namespace.to_string())?;

    // Graceful shutdown.
    docs.shutdown().await;
    let _ = blobs.shutdown().await;
    endpoint.close().await;

    Ok(namespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn init_persists_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let secret_key = SecretKey::generate();

        let namespace = init(path, &secret_key).await.unwrap();

        let hex = fs::read_to_string(path.join(NAMESPACE_FILE)).unwrap();
        assert_eq!(hex.len(), 64);
        let parsed: NamespaceId = hex.parse().unwrap();
        assert_eq!(parsed, namespace);
    }
}
