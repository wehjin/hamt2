use crate::{StorageBroadcastEvent, StorageService};
use serde::{Deserialize, Serialize};
use sky_trie::types::StorageHead;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    StorageStatus(StorageHead),
}

pub struct SocketHandler;
impl SocketHandler {
    pub async fn start(
        to_socket: Sender<SocketResponse>,
        mut from_socket: Receiver<SocketRequest>,
        storage: StorageService,
    ) -> Self {
        tokio::spawn(async move {
            let mut broadcasts = storage.subscribe();
            loop {
                tokio::select! {
                    Some(server_request) = from_socket.recv() => {
                        match server_request {
                            SocketRequest::Connect => {
                                let head  = storage.read_storage_head().await;
                                let _ = to_socket.send(SocketResponse::StorageStatus(head)).await;
                            }
                        }
                    }
                    Ok(event) = broadcasts.recv() => {
                        match event {
                            StorageBroadcastEvent::TransactFailed(_) => {}
                            StorageBroadcastEvent::NewHead(_) => {}
                        }
                    }
                }
            }
        });
        Self
    }
}

#[cfg(test)]
mod tests {
    use crate::{SocketHandler, SocketRequest, SocketResponse, StorageService};
    use sky_types::db::Attr;
    use sky_types::trie::SlotBaseId;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn socket_handler_works() {
        const COUNT: Attr = Attr("Counter/count");
        let storage = StorageService::start([COUNT]).await.unwrap();

        let (to_socket, mut from_handler) = mpsc::channel::<SocketResponse>(100);
        let (to_handler, from_socket) = mpsc::channel::<SocketRequest>(100);
        let _handler = SocketHandler::start(to_socket, from_socket, storage.clone()).await;
        {
            to_handler.send(SocketRequest::Connect).await.unwrap();
            let SocketResponse::StorageStatus(head) = from_handler.recv().await.unwrap();
            assert_ne!(SlotBaseId::ZERO, head.max_id);
        }
    }
}
