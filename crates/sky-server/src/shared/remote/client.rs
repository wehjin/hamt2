use crate::shared::remote::{RemoteClientReadStorage, SpawnTask};
use crate::shared::{SocketRequest, SocketResponse};
use anyhow::anyhow;
use sky_trie::prelude::ReadStorage;
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::{Datom, Transact, TransactError};
use sky_types::storage::ReadStorageError;
use sky_types::trie::{MapBase, SlotBaseId};
use std::collections::HashMap;
use std::marker::PhantomData;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub(crate) enum ClientRequest {
    DeliverHead(StorageHead),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
    DeliverBase(SlotBaseId, Option<SlotBase>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<StorageHead>>),
    DeliverTransact(StorageHead),
    Reconnect,
}

#[derive(Clone)]
pub struct ClientUpdater<T: SpawnTask> {
    requester: Sender<ClientRequest>,
    _phantom_data: PhantomData<T>,
}
impl<T: SpawnTask> ClientUpdater<T> {
    fn send_request(&self, request: ClientRequest) {
        send_request::<T>(&self.requester, request)
    }
    pub fn update(&mut self, socket_response: SocketResponse) {
        match socket_response {
            SocketResponse::StorageStatus(head) => {
                self.send_request(ClientRequest::DeliverHead(head));
            }
            SocketResponse::SlotBase(id, base) => {
                self.send_request(ClientRequest::DeliverBase(id, base));
            }
            SocketResponse::TransactResult(head) => {
                self.send_request(ClientRequest::DeliverTransact(head))
            }
        }
    }
}

/// Deliberately non-Clone.
pub struct RemoteClient<T: SpawnTask> {
    inner: RemoteClientReadStorage<T>,
    updater: ClientUpdater<T>,
}

impl<T: SpawnTask> ReadStorage for RemoteClient<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.inner.snapshot()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        self.inner.read(id).await
    }

    fn max_id(&self) -> SlotBaseId {
        self.inner.max_id()
    }

    fn read_root(&self) -> MapBase {
        self.inner.read_root()
    }
}

impl<T: SpawnTask> Transact for RemoteClient<T> {
    async fn transact(self, datoms: impl Into<Vec<Datom>>) -> Result<Self, TransactError>
    where
        Self: Sized,
    {
        match self.send_transact(datoms).await {
            Err(e) => Err(TransactError::Disconnected(e.into())),
            Ok(None) => Err(TransactError::Refused(anyhow!("Transaction failed"))),
            Ok(Some(head)) => {
                self.send_request(ClientRequest::DeliverHead(head));
                Ok(self)
            }
        }
    }
}

impl<T: SpawnTask> RemoteClient<T> {
    pub fn active_head(&self) -> StorageHead {
        self.inner.get_head()
    }

    pub fn to_updater(&self) -> ClientUpdater<T> {
        self.updater.clone()
    }
    pub fn update(&mut self, socket_response: SocketResponse) {
        self.updater.update(socket_response)
    }

    pub fn send_transact(
        &self,
        datoms: impl Into<Vec<Datom>>,
    ) -> oneshot::Receiver<Option<StorageHead>> {
        let (send, recv) = oneshot::channel();
        let request = ClientRequest::RequestTransact(datoms.into(), send);
        let _ = self.send_request(request);
        recv
    }

    pub fn reconnect(&self) {
        self.send_request(ClientRequest::Reconnect);
    }

    pub fn connect(send_socket: impl Fn(SocketRequest) + 'static) -> Self {
        let head = std::sync::Arc::new(std::sync::RwLock::new(StorageHead::default()));
        let (send_request, mut recv_request) = tokio::sync::mpsc::channel::<ClientRequest>(100);
        let send_socket = std::sync::Arc::new(send_socket);
        let task_send_socket = send_socket.clone();
        let task_head = head.clone();
        T::spawn_task(async move {
            let mut read_line: HashMap<SlotBaseId, Vec<oneshot::Sender<Option<SlotBase>>>> =
                HashMap::new();
            let mut bases = HashMap::from([(SlotBaseId::ZERO, SlotBase::new())]);
            let head = task_head;
            let mut transact_line: Option<oneshot::Sender<Option<StorageHead>>> = None;
            loop {
                let event = recv_request.recv().await;
                if let Some(request) = event {
                    match request {
                        ClientRequest::Reconnect => task_send_socket(SocketRequest::Connect),
                        ClientRequest::DeliverHead(new_head) => {
                            if new_head.max_id > head.read().unwrap().max_id {
                                let mut write_lock = head.write().unwrap();
                                *write_lock = new_head
                            }
                        }
                        ClientRequest::RequestBase(id, send_base) => {
                            if id > head.read().unwrap().max_id {
                                let _ = send_base.send(None);
                            } else {
                                if let Some(base) = bases.get(&id).cloned() {
                                    let _ = send_base.send(Some(base));
                                } else {
                                    let mut line = read_line.remove(&id).unwrap_or_default();
                                    line.push(send_base);
                                    read_line.insert(id, line);
                                    task_send_socket(SocketRequest::ReadSlotBase(id));
                                }
                            }
                        }
                        ClientRequest::DeliverBase(id, base) => {
                            let line = read_line.remove(&id).unwrap_or_default();
                            for send_base in line {
                                let base = base.clone();
                                let _ = send_base.send(base);
                            }
                            if let Some(base) = base.clone() {
                                bases.insert(id, base);
                            }
                        }
                        ClientRequest::RequestTransact(datoms, send_head) => {
                            if transact_line.is_some() {
                                let _ = send_head.send(None);
                            } else {
                                transact_line = Some(send_head);
                                task_send_socket(SocketRequest::Transact(datoms));
                            }
                        }
                        ClientRequest::DeliverTransact(new_head) => {
                            if let Some(send_head) = transact_line.take() {
                                let _ = send_head.send(Some(new_head));
                            }
                        }
                    }
                }
            }
        });
        send_socket(SocketRequest::Connect);
        let inner = RemoteClientReadStorage {
            requester: send_request.clone(),
            head,
            _spawn_local: PhantomData,
        };
        let updater = ClientUpdater {
            requester: send_request,
            _phantom_data: PhantomData,
        };
        Self { inner, updater }
    }

    fn send_request(&self, request: ClientRequest) {
        send_request::<T>(&self.inner.requester, request)
    }
}

fn send_request<T: SpawnTask>(requester: &Sender<ClientRequest>, request: ClientRequest) {
    let requester = requester.clone();
    T::spawn_task(async move {
        let _ = requester.send(request).await;
    })
}
