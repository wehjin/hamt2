use std::path::Path;
use thiserror::Error;

use iroh::{Endpoint, SecretKey, endpoint::presets, protocol::ProtocolHandler};
use iroh_blobs::Hash;
use iroh_blobs::api::blobs::Blobs;
use iroh_blobs::store::fs::FsStore;
use iroh_docs::AuthorId;
use iroh_docs::NamespaceId;
use iroh_docs::api::Doc;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_gossip::net::Gossip;

use crate::trie::{Base, Buffer, BufferIndex, BufferMut, KeyValue, MapBase, SkyTrie, SkyTrieMut, Slot};

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

#[derive(Error, Debug)]
pub enum BufferLoadError {
    #[error("{0}")]
    Anyhow(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum BufferPersistError {
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

    // Flush the blob store to disk, then shut down.
    docs.shutdown().await;
    blobs.sync_db().await?;
    let _ = blobs.shutdown().await;
    endpoint.close().await;

    Ok(namespace)
}

/// Loads the trie from the iroh-backed state at `path`.
///
/// Spins up the iroh constructs, reads the root from the doc, and walks the
/// trie fetching blobs to rebuild the in-memory [`SkyTrie`].
#[allow(dead_code)]
pub async fn load(path: &Path, secret_key: &SecretKey) -> Result<SkyTrie, BufferLoadError> {
    Ok(load_inner(path, secret_key).await?)
}

#[allow(dead_code)]
async fn load_inner(path: &Path, secret_key: &SecretKey) -> anyhow::Result<SkyTrie> {
    let endpoint = Endpoint::builder(presets::Minimal)
        .secret_key(secret_key.clone())
        .bind()
        .await?;

    let blobs = FsStore::load(path.join("blobs")).await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let docs = Docs::persistent(path.to_path_buf())
        .spawn(endpoint.clone(), blobs.clone().into(), gossip)
        .await?;

    // Read the namespace id, open the doc.
    let hex = std::fs::read_to_string(path.join(NAMESPACE_FILE))?;
    let namespace: NamespaceId = hex.parse()?;
    let doc = docs.open(namespace).await?.expect("doc not found");

    // Read the root hash, fetch + deserialize the root MapBase.
    let entry = doc
        .get_one(Query::key_exact(ROOT_KEY))
        .await?
        .expect("root not found");
    let root: MapBase = fetch_deserialize(blobs.blobs(), entry.content_hash()).await?;

    // Walk the trie, rebuilding via SkyTrieMut (re-hash on push).
    let mut edit = SkyTrieMut::new();
    load_map_base(root, &mut edit, blobs.blobs()).await?;
    edit.push_root(root).await;
    let trie = edit.commit().await;

    // Graceful shutdown.
    docs.shutdown().await;
    let _ = blobs.shutdown().await;
    endpoint.close().await;

    Ok(trie)
}

/// Recursively fetches and rebuilds the trie reachable from `mb`.
#[allow(dead_code)]
async fn load_map_base(mb: MapBase, edit: &mut SkyTrieMut, blobs: &Blobs) -> anyhow::Result<()> {
    // The empty base is a sentinel (NIL), not a real blob.
    if mb.base == BufferIndex::NIL {
        return Ok(());
    }
    let base: Base = fetch_deserialize(blobs, to_iroh_hash(mb.base)).await?;
    let index = edit.push_base(base.clone()).await;
    debug_assert_eq!(index, mb.base);

    for slot in &base.slots {
        match slot {
            Slot::KeyValue(kv) => match kv {
                KeyValue::Int { .. } => {}
                KeyValue::Subtrie { value, .. } => {
                    let sub_base: Base = fetch_deserialize(blobs, to_iroh_hash(*value)).await?;
                    let sub_mb = match &sub_base.slots[0] {
                        Slot::MapBase(mb) => *mb,
                        _ => anyhow::bail!("subtrie base should contain a map-base slot"),
                    };
                    edit.push_base(sub_base).await;
                    Box::pin(load_map_base(sub_mb, edit, blobs)).await?;
                }
                KeyValue::Bytes { value, .. } => {
                    let bytes = blobs.get_bytes(to_iroh_hash(*value)).await?;
                    edit.push_bytes(bytes).await;
                }
            },
            Slot::MapBase(sub) => Box::pin(load_map_base(*sub, edit, blobs)).await?,
        }
    }
    Ok(())
}

/// Persists the trie to the iroh-backed state.
///
/// Walks the trie from the root, writing each reachable base and byte array as
/// a blob, and records the root hash in the doc.
#[allow(dead_code)]
pub async fn persist(
    trie: &SkyTrie,
    blobs: &Blobs,
    doc: &Doc,
    author: AuthorId,
) -> Result<(), BufferPersistError> {
    Ok(persist_inner(trie, blobs, doc, author).await?)
}

#[allow(dead_code)]
async fn persist_inner(
    trie: &SkyTrie,
    blobs: &Blobs,
    doc: &Doc,
    author: AuthorId,
) -> anyhow::Result<()> {
    let root = trie.get_root();
    let root_hash = persist_map_base(root, trie, blobs).await?;
    let size = postcard::to_stdvec(&root)?.len() as u64;
    doc.set_hash(author, ROOT_KEY, root_hash, size).await?;
    Ok(())
}

/// Recursively writes the trie reachable from `mb`, returning the hash of the
/// serialized `mb` itself.
#[allow(dead_code)]
async fn persist_map_base(mb: MapBase, trie: &SkyTrie, blobs: &Blobs) -> anyhow::Result<Hash> {
    // The MapBase itself is a blob; its hash is what the doc (or a parent) points at.
    let mb_bytes = postcard::to_stdvec(&mb)?;
    let mb_hash = blobs.add_bytes(mb_bytes.clone()).temp_tag().await?.hash();

    // The empty base is a sentinel (NIL), not a real blob.
    if mb.base == BufferIndex::NIL {
        return Ok(mb_hash);
    }

    // The Base it points at is also a blob; its hash must equal mb.base.
    let base_slice = trie.get_base(mb.base, mb.map.slot_count());
    let base = Base { slots: base_slice.to_vec() };
    let base_bytes = postcard::to_stdvec(&base)?;
    let base_hash = blobs.add_bytes(base_bytes).temp_tag().await?.hash();
    debug_assert_eq!(base_hash, to_iroh_hash(mb.base));

    // Recurse into the slots.
    for slot in base_slice {
        match slot {
            Slot::KeyValue(kv) => match kv {
                KeyValue::Int { .. } => {}
                KeyValue::Subtrie { value, .. } => {
                    let sub = trie.get_subtrie(*value);
                    // Write the subtrie's base blob (a single MapBase slot).
                    let sub_base = Base { slots: vec![Slot::MapBase(sub)] };
                    let sub_base_bytes = postcard::to_stdvec(&sub_base)?;
                    let sub_base_hash = blobs.add_bytes(sub_base_bytes).temp_tag().await?.hash();
                    debug_assert_eq!(sub_base_hash, to_iroh_hash(*value));
                    Box::pin(persist_map_base(sub, trie, blobs)).await?;
                }
                KeyValue::Bytes { value, .. } => {
                    let _ = blobs.add_bytes(trie.get_bytes(*value)).temp_tag().await?;
                }
            },
            Slot::MapBase(sub) => {
                Box::pin(persist_map_base(*sub, trie, blobs)).await?;
            }
        }
    }
    Ok(mb_hash)
}

/// Fetches a blob and deserializes it.
async fn fetch_deserialize<T: serde::de::DeserializeOwned>(
    blobs: &Blobs,
    hash: Hash,
) -> anyhow::Result<T> {
    let bytes = blobs.get_bytes(hash).await?;
    Ok(postcard::from_bytes(&bytes)?)
}

/// Converts a [`BufferIndex`] (a BLAKE3 hash) to an iroh blob [`Hash`].
#[allow(dead_code)]
fn to_iroh_hash(index: BufferIndex) -> Hash {
    Hash::from(index.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trie::Query as _;
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

    #[tokio::test]
    async fn load_after_init_yields_empty_trie() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let secret_key = SecretKey::generate();

        init(path, &secret_key).await.unwrap();
        let trie = load(path, &secret_key).await.unwrap();

        assert_eq!(trie.get_root(), MapBase::empty());
        assert_eq!(trie.query_all(), vec![]);
    }

    #[tokio::test]
    async fn persist_then_load_round_trips() {
        use crate::trie::{Insert as _, InsertCursor as _, Query as _, TrieValue};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();
        let secret_key = SecretKey::generate();

        init(path, &secret_key).await.unwrap();

        // Build a trie with some data.
        let mut trie = SkyTrie::new();
        trie.edit(async |edit| {
            edit.insert(1, 42).await;
            edit.insert_deep([2, 3], 99, false).await;
            Ok(())
        })
        .await
        .unwrap();

        // Spin up constructs, persist.
        let endpoint = Endpoint::builder(presets::Minimal)
            .secret_key(secret_key.clone())
            .bind()
            .await
            .unwrap();
        let blobs = FsStore::load(path.join("blobs")).await.unwrap();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::persistent(path.to_path_buf())
            .spawn(endpoint.clone(), blobs.clone().into(), gossip)
            .await
            .unwrap();
        let hex = fs::read_to_string(path.join(NAMESPACE_FILE)).unwrap();
        let namespace: NamespaceId = hex.parse().unwrap();
        let doc = docs.open(namespace).await.unwrap().unwrap();
        let author = docs.author_default().await.unwrap();

        persist(&trie, blobs.blobs(), &doc, author).await.unwrap();

        docs.shutdown().await;
        let _ = blobs.shutdown().await;
        endpoint.close().await;

        // Load it back and check.
        let loaded = load(path, &secret_key).await.unwrap();
        assert_eq!(loaded.query_u32(1), Some(42));
        assert_eq!(loaded.query_deep([2, 3]), Some(TrieValue::U32(99)));
    }
}
