use std::path::Path;
use std::str::FromStr;

use data_encoding::HEXLOWER;
use iroh::SecretKey;
use iroh_docs::{DocTicket, NamespaceId};

use crate::_internal::log::Log;
use crate::_internal::rand_nonce;
use crate::_internal::replay;
use crate::attr_spec::DbSpec;
use crate::errors::ConnectError;
use crate::{datom, db, tx, Ein, Pod, Tx, Val};

const KEYPAIR_FILE: &str = "keypair";
const NAMESPACE_FILE: &str = "namespace";

fn load_or_create_keypair(path: &Path) -> Result<SecretKey, ConnectError> {
    let keypair_path = path.join(KEYPAIR_FILE);
    if keypair_path.exists() {
        let hex = std::fs::read_to_string(&keypair_path)?;
        let bytes = HEXLOWER
            .decode(hex.trim().as_bytes())
            .map_err(anyhow::Error::from)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("keypair must be 32 bytes"))?;
        Ok(SecretKey::from_bytes(&arr))
    } else {
        let key = SecretKey::generate();
        let hex = HEXLOWER.encode(&key.to_bytes());
        std::fs::write(&keypair_path, hex)?;
        Ok(key)
    }
}

fn persist_namespace(path: &Path, namespace: NamespaceId) -> Result<(), ConnectError> {
    std::fs::write(path.join(NAMESPACE_FILE), namespace.to_string())?;
    Ok(())
}

fn load_namespace(path: &Path) -> Result<NamespaceId, ConnectError> {
    let s = std::fs::read_to_string(path.join(NAMESPACE_FILE))?;
    Ok(NamespaceId::from_str(s.trim())?)
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time before epoch")
        .as_secs()
}

/// Creates a new repo: generates a keypair (if absent), creates the doc, writes the genesis
/// transaction, and persists the keypair + namespace id.
pub async fn create(
    path: impl AsRef<Path>,
    db_spec: impl Into<DbSpec>,
) -> Result<Pod, ConnectError> {
    let path = path.as_ref();
    std::fs::create_dir_all(path)?;
    let secret_key = load_or_create_keypair(path)?;
    let log = Log::create(secret_key, path).await?;
    let author = log.author();

    let db_spec = db_spec.into();
    let nonce = rand_nonce();
    let tx_ein = Ein::tx(&nonce);
    let mut genesis = Vec::new();
    for spec in db_spec.as_ref() {
        let ein = Ein::attr(&spec.attr);
        genesis.push(datom::add(ein, db::ident(), spec.attr.as_ident()));
        let card: Val = spec.cardinality.into();
        genesis.push(datom::add(ein, db::cardinality(), card));
    }
    genesis.push(datom::add(tx_ein, tx::nonce(), Val::Bytes(nonce.to_vec())));
    genesis.push(datom::add(
        tx_ein,
        tx::author(),
        Val::Bytes(author.to_bytes().to_vec()),
    ));
    genesis.push(datom::add(tx_ein, tx::instant(), Val::U64(now_secs())));
    let genesis_tx = Tx {
        prev: None,
        author,
        nonce,
        datoms: genesis,
    };

    log.append(&genesis_tx).await?;
    persist_namespace(path, log.namespace())?;

    let (schema, trie, order, tip) = replay::replay(vec![genesis_tx]).await?;
    Ok(Pod {
        schema,
        trie,
        order,
        tip,
        log,
    })
}

/// Loads an existing repo: reads the keypair + namespace id from the path, opens the doc, syncs,
/// and replays the log.
pub async fn load(path: impl AsRef<Path>) -> Result<Pod, ConnectError> {
    let path = path.as_ref();
    std::fs::create_dir_all(path)?;
    let secret_key = load_or_create_keypair(path)?;
    let namespace = load_namespace(path)?;
    let log = Log::open(secret_key, path, namespace).await?;

    let txs = log.entries().await?;
    let (schema, trie, order, tip) = replay::replay(txs).await?;
    Ok(Pod {
        schema,
        trie,
        order,
        tip,
        log,
    })
}

/// Joins a peer's repo: generates a keypair (if absent), imports the ticket, persists the
/// namespace id, syncs, and replays the log.
pub async fn join(path: impl AsRef<Path>, ticket: DocTicket) -> Result<Pod, ConnectError> {
    let path = path.as_ref();
    std::fs::create_dir_all(path)?;
    let secret_key = load_or_create_keypair(path)?;
    let log = Log::import(secret_key, path, ticket).await?;
    persist_namespace(path, log.namespace())?;

    let txs = log.entries().await?;
    let (schema, trie, order, tip) = replay::replay(txs).await?;
    Ok(Pod {
        schema,
        trie,
        order,
        tip,
        log,
    })
}
