use crate::shared::SocketResponse;
use crate::shared::remote::client::requests::ClientRequest;
use crate::shared::remote::{SpawnTask, client};
use std::marker::PhantomData;
use tokio::sync::mpsc::Sender;

#[derive(Clone)]
pub struct ClientUpdater<T: SpawnTask> {
    requester: Sender<ClientRequest>,
    _phantom_data: PhantomData<T>,
}

impl<T: SpawnTask> ClientUpdater<T> {
    pub(crate) fn new(requester: Sender<ClientRequest>) -> Self {
        Self {
            requester,
            _phantom_data: PhantomData,
        }
    }
    fn send_request(&self, request: ClientRequest) {
        client::send_request::<T>(&self.requester, request)
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
