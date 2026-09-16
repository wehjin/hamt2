//! Server layer for the sky-db stack.

use log::error;
use sky_db::ConnectError;
use sky_db::db::Db;
use sky_db::db::attr_spec::DbSpec;
use sky_db::storage::{MemDbStorage, ReadDbStorage};
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::{Datom, Transact, TransactError};
use sky_types::trie::SlotBaseId;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot};

pub struct StorageService {
    request_sender: mpsc::Sender<StorageRequest>,
    broadcast_receiver: broadcast::Receiver<StorageBroadcastEvent>,
}

impl StorageService {
    pub async fn start(db_spec: impl Into<DbSpec>) -> Result<Self, StorageServiceError> {
        let (request_sender, broadcast_receiver) = begin_storage(db_spec).await?;
        let service = Self {
            request_sender,
            broadcast_receiver,
        };
        Ok(service)
    }
    async fn transact(
        &self,
        datoms: impl Into<Vec<Datom>>,
    ) -> Result<StorageHead, StorageServiceError> {
        let datoms = datoms.into();
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::Transact(datoms, send);
        let _ = self.request_sender.send(request).await?;
        let new_storage_head = receive.await?;
        Ok(new_storage_head)
    }
}

#[derive(Error, Debug)]
pub enum StorageServiceError {
    #[error("Connection error: {0}")]
    ConnectError(#[from] ConnectError),

    #[error("Transport error: {0}")]
    TransactError(#[from] TransactError),

    #[error("Send storage request error: {0}")]
    SendStorageRequestError(#[from] mpsc::error::SendError<StorageRequest>),

    #[error("Receive storage response error: {0}")]
    ReceiveStorageResponseError(#[from] oneshot::error::RecvError),
}

#[derive(Debug, Clone)]
pub enum StorageBroadcastEvent {
    TransactFailed(String),
    NewHead(StorageHead),
}

#[derive(Debug)]
pub enum StorageRequest {
    Transact(Vec<Datom>, oneshot::Sender<StorageHead>),
    ReadSlotBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
}

pub async fn begin_storage(
    db_spec: impl Into<DbSpec>,
) -> Result<
    (
        mpsc::Sender<StorageRequest>,
        broadcast::Receiver<StorageBroadcastEvent>,
    ),
    StorageServiceError,
> {
    let db_spec = db_spec.into();
    let (to_clients, from_store) = broadcast::channel::<StorageBroadcastEvent>(1024);
    let (to_store, from_clients) = mpsc::channel::<StorageRequest>(1024);
    tokio::spawn(async move {
        let result = handle_storage(from_clients, to_clients, db_spec).await;
        if let Err(e) = result {
            error!("storage thread failed: {:?}", e);
        }
    });
    Ok((to_store, from_store))
}

async fn handle_storage(
    mut from_socks: mpsc::Receiver<StorageRequest>,
    to_socks: broadcast::Sender<StorageBroadcastEvent>,
    db_spec: DbSpec,
) -> Result<(), StorageServiceError> {
    let storage = MemDbStorage::new();
    let mut db = Db::new(storage.clone(), db_spec).await?;
    while let Some(event) = from_socks.recv().await {
        match event {
            StorageRequest::Transact(datoms, response) => match db.transact(datoms).await {
                Err(e) => {
                    let _ =
                        to_socks.send(StorageBroadcastEvent::TransactFailed(format!("{:?}", e)));
                    // For now, return an error because we've lost the db!!!
                    return Err(StorageServiceError::TransactError(e));
                }
                Ok(new_db) => {
                    let head = storage.get_head().await;
                    let broadcast = StorageBroadcastEvent::NewHead(head);
                    let _ = to_socks.send(broadcast);
                    let _ = response.send(head);
                    db = new_db;
                }
            },
            StorageRequest::ReadSlotBase(base_id, response) => {
                let base = storage.read(base_id).await.ok();
                let _ = response.send(base);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::StorageService;
    use sky_trie::types::StorageHead;
    use sky_types::db::{Attr, datum};

    #[tokio::test]
    async fn it_works() {
        const ATTR: Attr = Attr("Counter/count");
        let db_spec = [ATTR];
        let storage = StorageService::start(db_spec).await.unwrap();
        let new_head = storage.transact([datum::add(100, ATTR, 10)]).await.unwrap();
        let StorageHead { max_id, root } = new_head;
        assert_ne!(None, max_id);
        assert_ne!(None, root);
    }
}
