use crate::shared::remote::requests::ClientRequest;
use crate::shared::remote::updater::ClientUpdater;
use crate::shared::remote::{RemoteClientReadStorage, SpawnTask};
use crate::shared::{SocketRequest, SocketResponse};
use sky_trie::prelude::ReadStorage;
use sky_trie::types::StorageHead;
use sky_types::db::Datom;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};

pub mod handler;
pub mod read_storage;
pub mod requests;
pub mod transact;
pub mod updater;

/// Deliberately non-Clone.
pub struct RemoteClient<T: SpawnTask> {
    inner: RemoteClientReadStorage<T>,
    updater: ClientUpdater<T>,
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
        let head = Arc::new(RwLock::new(StorageHead::default()));
        let (send_request, recv_request) = mpsc::channel::<ClientRequest>(100);
        let send_socket = Arc::new(send_socket);
        let task_send_socket = send_socket.clone();
        let task_head = head.clone();
        T::spawn_task(handler::process_client_requests(
            recv_request,
            task_send_socket,
            task_head,
        ));
        send_socket(SocketRequest::Connect);
        let inner = RemoteClientReadStorage {
            requester: send_request.clone(),
            head,
            _spawn_local: PhantomData,
        };
        let updater = ClientUpdater::new(send_request);
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
