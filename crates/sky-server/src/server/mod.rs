use crate::server::storage::{StorageBroadcastEvent, StorageService};
use crate::shared::{SocketRequest, SocketResponse};
use futures_util::{Sink, SinkExt, Stream, StreamExt};

#[cfg(feature = "axum-ws")]
pub mod axum_ws;
pub mod storage;

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
            request = incoming.next() => {
                match request {
                    Some(SocketRequest::Connect) => {
                        let status  = storage.read_status().await;
                        let _ = outgoing.send(SocketResponse::DbStatus(status)).await;
                    }
                    Some(SocketRequest::ReadSlotBase(slotbase_id)) => {
                        let slot_base = storage.read_slot_base(slotbase_id).await;
                        let _ = outgoing.send(SocketResponse::SlotBase(slotbase_id, slot_base)).await;
                    }
                    Some(SocketRequest::Transact(datoms)) => {
                        // Failures are logged by the storage task and never
                        // reach the socket protocol (yet).
                        if let Ok(head) = storage.transact(datoms).await {
                            let _ = outgoing.send(SocketResponse::TransactResult(head)).await;
                        }
                    }
                    // The client disconnected: end the connection task.
                    None => break,
                }
            }
            Ok(event) = broadcast.recv() => {
                let StorageBroadcastEvent::NewStatus(status) = event;
                let _ = outgoing.send(SocketResponse::DbStatus(status)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::server::process_socket_requests;
    use crate::server::storage::StorageService;
    use crate::shared::{SocketRequest, SocketResponse};
    use sky_types::db::{Attr, datom};
    use sky_types::trie::BaseId;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::task::JoinHandle;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_util::sync::PollSender;

    #[tokio::test]
    async fn process_socket_requests_works() {
        let storage = StorageService::start(["Counter/count"]).await.unwrap();
        let (_task, request, mut response) = spawn_socket_task(storage);

        // Connect
        let status = {
            request.send(SocketRequest::Connect).await.unwrap();
            let SocketResponse::DbStatus(status) = response.recv().await.unwrap() else {
                panic!("Unexpected response received");
            };
            assert_ne!(BaseId::ZERO, status.head.max_id);
            status
        };

        // Read slot base
        {
            request
                .send(SocketRequest::ReadSlotBase(status.head.max_id))
                .await
                .unwrap();
            let SocketResponse::SlotBase(base_id, Some(base)) = response.recv().await.unwrap()
            else {
                panic!("Unexpected response received");
            };
            assert_eq!(status.head.max_id, base_id);
            assert_ne!(0, base.len());
        }
    }

    #[tokio::test]
    async fn transact_over_socket_reaches_clients() {
        let attr = || Attr::from("Counter/count");
        let storage = StorageService::start([attr()]).await.unwrap();
        let (_task, request, mut response) = spawn_socket_task(storage);

        request
            .send(SocketRequest::Transact(vec![datom::add(100, attr(), 10)]))
            .await
            .unwrap();

        // The client receives the direct transact result plus the forwarded
        // NewHead broadcast; their order depends on select scheduling.
        let first = response.recv().await.unwrap();
        let second = response.recv().await.unwrap();
        let mut result_heads = Vec::new();
        let mut status_heads = Vec::new();
        for response in [first, second] {
            match response {
                SocketResponse::TransactResult(status) => result_heads.push(status.head),
                SocketResponse::DbStatus(status) => status_heads.push(status.head),
                other => panic!("Unexpected response: {:?}", other),
            }
        }
        assert_eq!(1, result_heads.len());
        assert_eq!(1, status_heads.len());
        assert_eq!(result_heads[0], status_heads[0]);
        assert_ne!(BaseId::ZERO, result_heads[0].max_id);
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
