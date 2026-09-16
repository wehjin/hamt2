//! Server layer for the sky-db stack.

mod storage;

use futures_util::{Sink, SinkExt, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::trie::SlotBaseId;
pub use storage::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
    ReadSlotBase(SlotBaseId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    StorageStatus(StorageHead),
    SlotBase(SlotBaseId, Option<SlotBase>),
}

/// Processes socket requests in a loop.
///
/// We use Streams and Sinks so in the parameters so that the logic can be tested here
/// but also used in other crates for example to process WebSockets.
pub async fn process_socket_requests<In, Out>(
    mut incoming: In,
    mut outgoing: Out,
    storage: StorageService,
) where
    In: Stream<Item = SocketRequest> + Unpin,
    Out: Sink<SocketResponse> + Unpin,
{
    let mut broadcast = storage.subscribe();
    loop {
        tokio::select! {
            Some(request) = incoming.next() => {
                match request {
                    SocketRequest::Connect => {
                        let head  = storage.read_storage_head().await;
                        let _ = outgoing.send(SocketResponse::StorageStatus(head)).await;
                    }
                    SocketRequest::ReadSlotBase(slotbase_id) => {
                        let slot_base = storage.read_slot_base(slotbase_id).await;
                        let _ = outgoing.send(SocketResponse::SlotBase(slotbase_id, slot_base)).await;
                    }
                }
            }
            Ok(event) = broadcast.recv() => {
                match event {
                    StorageBroadcastEvent::TransactFailed(_) => {}
                    StorageBroadcastEvent::NewHead(_) => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::StorageService;
    use crate::{SocketRequest, SocketResponse, process_socket_requests};
    use sky_types::db::Attr;
    use sky_types::trie::SlotBaseId;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::task::JoinHandle;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;

    #[tokio::test]
    async fn process_socket_requests_works() {
        const COUNT: Attr = Attr("Counter/count");
        let storage = StorageService::start([COUNT]).await.unwrap();
        let (_task, request, mut response) = spawn_socket_task(storage);

        // Connect
        let head = {
            request.send(SocketRequest::Connect).await.unwrap();
            let SocketResponse::StorageStatus(head) = response.recv().await.unwrap() else {
                panic!("Unexpected response received");
            };
            assert_ne!(SlotBaseId::ZERO, head.max_id);
            head
        };

        // Read slot base
        {
            request
                .send(SocketRequest::ReadSlotBase(head.max_id))
                .await
                .unwrap();
            let SocketResponse::SlotBase(base_id, Some(base)) = response.recv().await.unwrap()
            else {
                panic!("Unexpected response received");
            };
            assert_eq!(head.max_id, base_id);
            assert_ne!(0, base.len());
        }
    }

    fn spawn_socket_task(
        storage: StorageService,
    ) -> (
        JoinHandle<()>,
        Sender<SocketRequest>,
        Receiver<SocketResponse>,
    ) {
        let (send_response, response) = mpsc::channel::<SocketResponse>(10);
        let (request, receive_request) = mpsc::channel::<SocketRequest>(10);
        let task = tokio::spawn(process_socket_requests(
            ReceiverStream::new(receive_request),
            PollSender::new(send_response),
            storage.clone(),
        ));
        (task, request, response)
    }
}
