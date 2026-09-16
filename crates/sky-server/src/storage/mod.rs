use log::error;
use sky_db::db::Db;
use sky_db::db::attr_spec::DbSpec;
use sky_trie::prelude::{MemTrieStorage as MemDbStorage, ReadStorage};
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::{Datom, Transact};
use sky_types::trie::SlotBaseId;
use tokio::sync::{broadcast, mpsc, oneshot};
mod types;

pub use types::*;

/// The storage service is started once. Later, multiple tasks can
/// connect to it and pass messages.
#[derive(Clone)]
pub struct StorageService {
    request_sender: mpsc::Sender<StorageRequest>,
    /// We do not broadcast events. We have the sender
    /// only to subscribe to it when needed.
    broadcast_sender: broadcast::Sender<StorageBroadcastEvent>,
}

/// The requester is used to send messages to the storage service.
#[derive(Clone)]
pub struct StorageRequester {
    request_sender: mpsc::Sender<StorageRequest>,
}

impl StorageRequester {
    pub async fn transact(
        &self,
        datoms: impl Into<Vec<Datom>>,
    ) -> Result<StorageHead, StorageServiceError> {
        let datoms = datoms.into();
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::Transact(datoms, send);
        self.request_sender
            .send(request)
            .await
            .expect("send transact failed");
        let new_storage_head = receive.await.expect("receive transact response failed");
        Ok(new_storage_head)
    }

    pub async fn read_slot_base(
        &self,
        slot_base_id: SlotBaseId,
    ) -> Result<Option<SlotBase>, StorageServiceError> {
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::ReadSlotBase(slot_base_id, send);
        self.request_sender
            .send(request)
            .await
            .expect("send read-slot-base failed");
        let slot_base = receive
            .await
            .expect("receive read-slot-base response failed");
        Ok(slot_base)
    }
}

/// These are messages sent to the storage service after connection.
#[derive(Debug)]
enum StorageRequest {
    Transact(Vec<Datom>, oneshot::Sender<StorageHead>),
    ReadSlotBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
}

impl StorageService {
    pub async fn start(db_spec: impl Into<DbSpec>) -> Result<Self, StorageServiceError> {
        let (request_sender, broadcast_sender) = begin_storage(db_spec).await?;
        let service = Self {
            request_sender,
            broadcast_sender,
        };
        Ok(service)
    }

    /// Connecting to the storage service produces a requester and a broadcast receiver.
    /// The requester is used to transact directly with the service, while the receiver
    /// must be used to listen to messages from the service.  If it is left unread and
    /// undropped, it will accumulate broadcast messages and eventually overflow.
    pub fn subscribe(&self) -> (StorageRequester, broadcast::Receiver<StorageBroadcastEvent>) {
        let requester = StorageRequester {
            request_sender: self.request_sender.clone(),
        };
        let receiver = self.broadcast_sender.subscribe();
        (requester, receiver)
    }
}

async fn begin_storage(
    db_spec: impl Into<DbSpec>,
) -> Result<
    (
        mpsc::Sender<StorageRequest>,
        broadcast::Sender<StorageBroadcastEvent>,
    ),
    StorageServiceError,
> {
    let db_spec = db_spec.into();
    let (to_clients, _from_task) = broadcast::channel::<StorageBroadcastEvent>(1024);
    let (to_task, from_clients) = mpsc::channel::<StorageRequest>(1024);
    let result = (to_task, to_clients.clone());
    tokio::spawn(async move {
        let result = handle_storage(from_clients, to_clients, db_spec).await;
        if let Err(e) = result {
            error!("storage thread failed: {:?}", e);
        }
    });
    Ok(result)
}

async fn handle_storage(
    mut from_clients: mpsc::Receiver<StorageRequest>,
    to_clients: broadcast::Sender<StorageBroadcastEvent>,
    db_spec: DbSpec,
) -> Result<(), StorageServiceError> {
    let storage = MemDbStorage::new();
    let mut db = Db::new(storage.clone(), db_spec).await?;
    while let Some(event) = from_clients.recv().await {
        match event {
            StorageRequest::Transact(datoms, response) => match db.transact(datoms).await {
                Err(e) => {
                    let _ =
                        to_clients.send(StorageBroadcastEvent::TransactFailed(format!("{:?}", e)));
                    // For now, return an error because we've lost the db!!!
                    return Err(StorageServiceError::TransactError(e));
                }
                Ok(new_db) => {
                    let head = storage.get_head().await;
                    let broadcast = StorageBroadcastEvent::NewHead(head);
                    let _ = to_clients.send(broadcast);
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
    use crate::storage::StorageService;
    use crate::storage::types::StorageBroadcastEvent;
    use sky_trie::types::StorageHead;
    use sky_types::db::{Attr, datom};

    #[tokio::test]
    async fn it_works() {
        const ATTR: Attr = Attr("Counter/count");
        let db_spec = [ATTR];
        let storage = StorageService::start(db_spec).await.unwrap();

        let (requester, mut receiver) = storage.subscribe();

        let new_head = requester
            .transact([datom::add(100, ATTR, 10)])
            .await
            .unwrap();
        let StorageHead { max_id, root } = new_head;
        assert_ne!(None, max_id);
        assert_ne!(None, root);

        let broadcast = receiver.recv().await.unwrap();
        assert_eq!(
            StorageBroadcastEvent::NewHead(StorageHead { max_id, root }),
            broadcast
        );

        let max_id = max_id.unwrap();
        let slot_base = requester.read_slot_base(max_id).await.unwrap();
        assert_ne!(None, slot_base);
    }
}
