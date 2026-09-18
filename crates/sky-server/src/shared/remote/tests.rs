use super::*;
use crate::shared::{SocketRequest, SocketResponse};
use sky_trie::storage::ReadStorage;
use sky_trie::types::StorageHead;
use sky_trie::types::slot::Slot;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::{Attr, Transact, datom, val};
use sky_types::trie::{MapBase, SlotBaseId, SlotMap, TrieValue};
use std::time::Duration;
use tokio::task::spawn_local;

async fn run_client_test<F, Fut>(runner: F)
where
    F: FnOnce(RemoteClient<TokioSpawnLocal>, tokio::sync::mpsc::Receiver<SocketRequest>) -> Fut,
    Fut: Future,
{
    tokio::task::LocalSet::new()
        .run_until(async move {
            // Set up fake socket.
            let (tokio_send_socket, mut requests_from_client) =
                tokio::sync::mpsc::channel::<SocketRequest>(100);

            // Connect the client and check its first emission.
            let client = {
                let client_send_socket = move |request| {
                    let send_request = tokio_send_socket.clone();
                    tokio::task::spawn_local(async move {
                        send_request.send(request).await.expect("sending request");
                    });
                };
                RemoteClient::<TokioSpawnLocal>::connect(client_send_socket)
            };
            let client_request_after_connect =
                requests_from_client.recv().await.expect("recv request");
            assert_eq!(client_request_after_connect, SocketRequest::Connect);
            runner(client, requests_from_client).await;
        })
        .await;
}

#[tokio::test]
async fn transact_works() {
    run_client_test(|client, mut requests_from_client| async move {
        // Transact with the client and check its emission is a transmit request. The call
        // to transact must be in another task so that we can continue working before the
        // transact call returns.
        let datom = datom::add(20, Attr::from("jp/hello"), val("konnichiwa"));
        let txn_datom = datom.clone();
        let mut updater = client.to_updater();
        let join =
            spawn_local(async move { client.transact([txn_datom]).await.expect("transact") });
        let emission = requests_from_client.recv().await.expect("recv request");
        assert_eq!(SocketRequest::Transact(vec![datom]), emission);

        // Transact waits for an acknowledgement before returning so we feed it one. After
        // that, the client should return from the transact call.
        let new_head = StorageHead {
            max_id: SlotBaseId(1),
            root: MapBase {
                map: Default::default(),
                base: Default::default(),
            },
        };
        updater.update(SocketResponse::TransactResult(new_head));
        let result = tokio::time::timeout(Duration::from_secs(1), join)
            .await
            .expect("join");
        assert!(result.is_ok(), "join after transact timed out");
    })
    .await
}

#[tokio::test]
async fn remote_client_works() {
    run_client_test(|mut client, mut requests_from_client| async move {
        // Update the client with the first response from the socket.
        let id1 = SlotBaseId(1);
        let first_socket_response = SocketResponse::StorageStatus(StorageHead {
            max_id: id1,
            root: MapBase {
                map: SlotMap::empty(),
                base: id1,
            },
        });
        client.update(first_socket_response);
        let active_head = client.active_head().await.expect("active head");
        assert_eq!(active_head.max_id, id1);

        // Start a read at the client and check it sent a request to the socket. The read must
        // be in a separate call so we can continue working before the read returns.
        let mut updater = client.to_updater();
        let join = tokio::task::spawn_local(async move {
            let read = client.read(id1).await.unwrap();
            (client, read)
        });
        let client_request_after_read = requests_from_client.recv().await.expect("recv request");
        assert_eq!(client_request_after_read, SocketRequest::ReadSlotBase(id1));

        // Deliver the read to the client and check it comes back out.
        let fed_to_client = SlotBase::new().insert_slot(0, Slot::KeyValue(1, TrieValue::U32(15)));
        updater.update(SocketResponse::SlotBase(id1, Some(fed_to_client.clone())));
        let join_result = tokio::time::timeout(Duration::from_secs(1), join)
            .await
            .expect("join");
        let Ok((client, read_from_client)) = join_result else {
            panic!("join after update timed out");
        };
        assert_eq!(fed_to_client, read_from_client);

        // Try the read again. This time it should be in the cache and there should be
        // no request sent to socket.
        let read_result = tokio::time::timeout(Duration::from_secs(1), client.read(id1))
            .await
            .expect("read");
        let Ok(second_read_from_client) = read_result else {
            panic!("join after read timed out");
        };
        assert_eq!(fed_to_client, second_read_from_client);
        assert!(requests_from_client.is_empty());
    })
    .await
}

#[derive(Copy, Clone)]
struct TokioSpawnLocal;
impl SpawnLocal for TokioSpawnLocal {
    fn spawn_local(future: impl Future<Output = ()> + 'static) {
        tokio::task::spawn_local(future);
    }
}
