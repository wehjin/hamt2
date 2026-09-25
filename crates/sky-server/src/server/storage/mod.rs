use log::error;
use sky_db::db::Db;
use sky_db::db::attr_spec::DbSpec;
use sky_types::db::{Datom, DbStatus, Transact};
use sky_types::storage::mem_edit_new;
use sky_types::trie::{Base, BaseId};
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
    pub async fn read_status(&self) -> DbStatus {
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::ReadStatus(send);
        self.request_sender
            .send(request)
            .await
            .expect("send request failed");
        receive.await.expect("receive response failed")
    }

    /// Reads a slot base from the storage service. Returns `None` for ids
    /// outside the current head (negative or beyond `max_id`).
    pub async fn read_slot_base(&self, slot_base_id: BaseId) -> Option<Base> {
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
    ) -> Result<DbStatus, StorageServiceError> {
        let datoms = datoms.into();
        let (send, receive) = oneshot::channel();
        let request = StorageRequest::Transact(datoms, send);
        self.request_sender
            .send(request)
            .await
            .expect("send request failed");
        match receive.await {
            Ok(status) => Ok(status),
            // The storage task dropped the sender unsent: the transact lost
            // the db and the worker exited.
            Err(_) => Err(StorageServiceError::TransactFailed),
        }
    }
}

/// These are messages sent to the storage service after connection.
#[derive(Debug)]
enum StorageRequest {
    ReadStatus(oneshot::Sender<DbStatus>),
    ReadSlotBase(BaseId, oneshot::Sender<Option<Base>>),
    Transact(Vec<Datom>, oneshot::Sender<DbStatus>),
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
    let storage = mem_edit_new();
    let mut db = Db::new(storage, db_spec).await?;
    while let Some(event) = from_clients.recv().await {
        match event {
            StorageRequest::ReadStatus(response) => {
                let head = db.status();
                let schema = db.schema().clone();
                let status = DbStatus { head, schema };
                if let Err(e) = response.send(status) {
                    error!("Failed to send status: {:?}", e);
                }
            }
            StorageRequest::ReadSlotBase(base_id, response) => {
                // Guard against ids the storage has never assigned; reading
                // them directly would panic the storage task.
                let base = if base_id < BaseId::ZERO || base_id > db.status().max_id {
                    None
                } else {
                    db.read_base(base_id).await.ok()
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
                    let head = new_db.status();
                    let schema = new_db.schema().clone();
                    let status = DbStatus { head, schema };
                    let broadcast = StorageBroadcastEvent::NewStatus(status.clone());
                    let _ = to_clients.send(broadcast);
                    let _ = response.send(status);
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
    use sky_types::db::{Attr, DbStatus, datom};
    use sky_types::storage::StorageStatus;
    use sky_types::trie::{BaseId, MapBase};

    #[tokio::test]
    async fn it_works() {
        let attr = || Attr::from("Counter/count");
        let db_spec = [attr()];
        let storage = StorageService::start(db_spec).await.unwrap();
        let status = storage.read_status().await;
        let mut broadcasts = storage.subscribe();

        let new_status = storage
            .transact([datom::add(100, attr(), 10)])
            .await
            .unwrap();
        let StorageStatus { max_id, root } = new_status.head;
        assert_ne!(BaseId::ZERO, max_id);
        assert_ne!(MapBase::empty(), root);

        let broadcast = broadcasts.recv().await.unwrap();
        assert_eq!(
            broadcast,
            StorageBroadcastEvent::NewStatus(DbStatus {
                head: StorageStatus { max_id, root },
                schema: status.schema.clone(),
            }),
        );

        let slot_base = storage.read_slot_base(max_id).await;
        assert_ne!(None, slot_base);

        let out_of_range = storage.read_slot_base(BaseId(10_000)).await;
        assert_eq!(None, out_of_range);
    }
}
