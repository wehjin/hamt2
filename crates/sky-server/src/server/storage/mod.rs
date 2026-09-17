use log::error;
use sky_db::db::Db;
use sky_db::db::attr_spec::DbSpec;
use sky_trie::prelude::{MemStorage, ReadStorage};
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

impl StorageService {
    /// Starts the storage service
    pub async fn start(db_spec: impl Into<DbSpec>) -> Result<Self, StorageServiceError> {
        let (request_sender, broadcast_sender) = begin_storage(db_spec).await?;
        let service = Self {
            request_sender,
            broadcast_sender,
        };
        Ok(service)
    }

    /// Produces a broadcast-event receiver for the service's broadcast events.
    ///
    /// # Gotchas
    /// The receiver must be read in a loop or dropped. If it is left unread and undropped, it
    /// will accumulate broadcast messages and eventually overflow.
    pub fn subscribe(&self) -> broadcast::Receiver<StorageBroadcastEvent> {
        self.broadcast_sender.subscribe()
    }

    /// Read the head of the storage service. This tells the client what is in
    /// the storage.
    pub async fn read_storage_head(&self) -> StorageHead {
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::ReadStorageHead(send);
        self.request_sender
            .send(request)
            .await
            .expect("send request failed");
        receive.await.expect("receive response failed")
    }

    /// Reads a slot base from the storage service. Returns `None` for ids
    /// outside the current head (negative or beyond `max_id`).
    pub async fn read_slot_base(&self, slot_base_id: SlotBaseId) -> Option<SlotBase> {
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::ReadSlotBase(slot_base_id, send);
        self.request_sender
            .send(request)
            .await
            .expect("send request failed");
        receive.await.expect("receive response failed")
    }

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
            .expect("send request failed");
        match receive.await {
            Ok(head) => Ok(head),
            // The storage task dropped the sender unsent: the transact lost
            // the db and the worker exited.
            Err(_) => Err(StorageServiceError::TransactFailed),
        }
    }
}

/// These are messages sent to the storage service after connection.
#[derive(Debug)]
enum StorageRequest {
    ReadStorageHead(oneshot::Sender<StorageHead>),
    ReadSlotBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
    Transact(Vec<Datom>, oneshot::Sender<StorageHead>),
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
    let storage = MemStorage::new();
    let mut db = Db::new(storage.clone(), db_spec).await?;
    while let Some(event) = from_clients.recv().await {
        match event {
            StorageRequest::ReadStorageHead(response) => {
                let head = storage.get_head();
                if let Err(e) = response.send(head) {
                    log::error!("ReadTrieStatus response failed: {:?}", e);
                }
            }
            StorageRequest::ReadSlotBase(base_id, response) => {
                // Guard against ids the storage has never assigned; reading
                // them directly would panic the storage task.
                let base = if base_id < SlotBaseId::ZERO || base_id > storage.max_id() {
                    None
                } else {
                    storage.read(base_id).await.ok()
                };
                if let Err(e) = response.send(base) {
                    error!("ReadSlotBase response failed: {:?}", e);
                }
            }
            StorageRequest::Transact(datoms, response) => match db.transact(datoms).await {
                Err(e) => {
                    error!("transact failed, storage stopping: {:?}", e);
                    // For now, return an error because we've lost the db!!!
                    return Err(StorageServiceError::TransactError(e));
                }
                Ok(new_db) => {
                    let head = storage.get_head();
                    let broadcast = StorageBroadcastEvent::NewHead(head);
                    let _ = to_clients.send(broadcast);
                    let _ = response.send(head);
                    db = new_db;
                }
            },
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
	use crate::server::storage::StorageService;
	use crate::server::storage::types::StorageBroadcastEvent;
	use sky_trie::types::StorageHead;
	use sky_types::db::{Attr, datom};
	use sky_types::trie::{MapBase, SlotBaseId};

	#[tokio::test]
	async fn it_works() {
        let attr = || Attr::from("Counter/count");
        let db_spec = [attr()];
        let storage = StorageService::start(db_spec).await.unwrap();
        let mut broadcasts = storage.subscribe();

        let new_head = storage.transact([datom::add(100, attr(), 10)]).await.unwrap();
        let StorageHead { max_id, root } = new_head;
        assert_ne!(SlotBaseId::ZERO, max_id);
        assert_ne!(MapBase::empty(), root);

        let broadcast = broadcasts.recv().await.unwrap();
        assert_eq!(
            StorageBroadcastEvent::NewHead(StorageHead { max_id, root }),
            broadcast
        );

        let slot_base = storage.read_slot_base(max_id).await;
        assert_ne!(None, slot_base);

        let out_of_range = storage.read_slot_base(SlotBaseId(10_000)).await;
        assert_eq!(None, out_of_range);
    }
}
