use std::sync::Arc;

use futures::StreamExt;
use iroh_docs::DocTicket;
use iroh_docs::engine::LiveEvent;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use crate::_internal::log::Log;
use crate::_internal::replay::{self, Index, apply_tx};
use crate::_internal::{EntEid, now_secs, rand_nonce};
use crate::errors::ConnectError;
use crate::{Dat, Datom, Ein, Ent, Pod, Transact, TransactError, Tx, Val, datom, tx, val};

/// A live peer: owns the log and the shared index, and replays incoming transactions in the
/// background. Hand out stable `Pod` snapshots via [`Peer::snapshot`].
pub struct Peer {
    log: Log,
    index: Arc<RwLock<Index>>,
    task: JoinHandle<()>,
}

impl Peer {
    pub(crate) fn new(log: Log, index: Index) -> Self {
        let index = Arc::new(RwLock::new(index));
        let task = spawn_background(log.clone(), index.clone());
        Self { log, index, task }
    }

    /// A stable, immutable snapshot of the current index.
    pub async fn snapshot(&self) -> Pod {
        let index = self.index.read().await;
        Pod {
            schema: index.schema.clone(),
            trie: index.trie.clone(),
            order: index.order.clone(),
        }
    }

    /// Recreates a fresh ticket for this peer's log (with the current endpoint address).
    pub async fn ticket(&self) -> Result<DocTicket, ConnectError> {
        Ok(self.log.ticket().await?)
    }
}

impl Transact for Peer {
    async fn transact(
        &mut self,
        datoms: impl Into<Vec<Datom>>,
    ) -> Result<&mut Self, TransactError> {
        let datoms = datoms.into();
        if datoms.is_empty() {
            return Ok(self);
        }
        let nonce = rand_nonce();
        let tx_ein = Ein::tx(&nonce);
        // Resolve temp entities and build the full datom set.
        let ent_eid = EntEid::new(&datoms);
        let mut all = Vec::new();
        for datom in datoms {
            let eid = match &datom.ent {
                Ent::Id(eid) => *eid,
                Ent::Temp(name) => ent_eid[name.as_str()],
            };
            let attr = datom.attr;
            let val = match datom.dat {
                Dat::Val(val) => val,
                Dat::Ent(ent) => {
                    let eid = match ent {
                        Ent::Id(eid) => eid,
                        Ent::Temp(name) => ent_eid[name.as_str()],
                    };
                    val(eid)
                }
            };
            all.push(Datom {
                ent: Ent::Id(eid),
                attr,
                dat: Dat::Val(val),
                dir: datom.dir,
            });
        }
        all.push(datom::add(tx_ein, tx::nonce(), Val::Bytes(nonce.to_vec())));
        let author = self.log.author();
        all.push(datom::add(
            tx_ein,
            tx::author(),
            Val::Bytes(author.to_bytes().to_vec()),
        ));
        all.push(datom::add(tx_ein, tx::instant(), Val::U64(now_secs())));
        let tip = self.index.read().await.tip;
        let tx = Tx {
            prev: tip,
            author,
            nonce,
            datoms: all,
        };
        let hash = tx.hash();
        // Append to the log, then apply to the local index.
        self.log.append(&tx).await.map_err(TransactError::Rewind)?;
        {
            let mut index = self.index.write().await;
            let Index {
                schema,
                trie,
                order,
                tip,
            } = &mut *index;
            apply_tx(trie, schema, &tx, tx_ein).await?;
            let pos = order.len() as u32;
            order.insert(tx_ein, pos);
            *tip = Some(hash);
        }
        Ok(self)
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn spawn_background(log: Log, index: Arc<RwLock<Index>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut events = match log.subscribe().await {
            Ok(events) => events,
            Err(_) => return,
        };
        while let Some(event) = events.next().await {
            let Ok(event) = event else { continue };
            match event {
                LiveEvent::ContentReady { .. }
                | LiveEvent::PendingContentReady
                | LiveEvent::SyncFinished(_) => {
                    let Ok(txs) = log.entries().await else { continue };
                    let Ok(new_index) = replay::replay(txs).await else { continue };
                    *index.write().await = new_index;
                }
                _ => {}
            }
        }
    })
}
