use super::*;
use crate::shared::{SocketRequest, SocketResponse};
use sky_db::traits::DbQuery;
use sky_types::db;
use sky_types::db::schema::Schema;
use sky_types::db::{Attr, DbStatus, Transact, datom, val};
use sky_types::storage::StorageStatus;
use sky_types::trie::{MapBase, Slot, Base, BaseId, SlotMap, RootBaseRead, TrieValue};
use std::time::Duration;
use tokio::task::spawn_local;

#[tokio::test]
async fn get_reader_works() {
    run_client_test(|mut client, mut socket_requests| async move {
        let db_status = DbStatus {
            head: StorageStatus {
                max_id: BaseId(10),
                root: MapBase {
                    map: SlotMap(0xffffffff),
                    base: BaseId(1),
                },
            },
            schema: Schema::default(),
        };
        client.update(SocketResponse::DbStatus(db_status));
        tokio::task::yield_now().await;
        let reader = client.to_reader();
        spawn_local(async move { reader.find_val(0, db::ident()).await });
        tokio::task::yield_now().await;
        let mut contents = Vec::new();
        while let Ok(msg) = socket_requests.try_recv() {
            contents.push(msg);
        }
        assert_eq!(SocketRequest::ReadSlotBase(BaseId(1)), contents[0]);
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
        let status = DbStatus {
            head: StorageStatus {
                max_id: BaseId(1),
                root: MapBase {
                    map: Default::default(),
                    base: Default::default(),
                },
            },
            schema: Schema::default(),
        };
        updater.update(SocketResponse::TransactResult(status));
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
        let id1 = BaseId(1);
        let first_socket_response = SocketResponse::DbStatus(DbStatus {
            head: StorageStatus {
                max_id: id1,
                root: MapBase {
                    map: SlotMap::empty(),
                    base: id1,
                },
            },
            schema: Schema::starter(),
        });
        client.update(first_socket_response);
        tokio::task::yield_now().await;
        assert_eq!(id1, client.active_head().max_id);

        // Start a read at the client and check it sent a request to the socket. The read must
        // be in a separate call so we can continue working before the read returns.
        let mut updater = client.to_updater();
        let join = spawn_local(async move {
            let read = client.read_base(id1).await.unwrap();
            (client, read)
        });
        let client_request_after_read = requests_from_client.recv().await.expect("recv request");
        assert_eq!(client_request_after_read, SocketRequest::ReadSlotBase(id1));

        // Deliver the read to the client and check it comes back out.
        let fed_to_client = Base::empty().insert_slot(0, Slot::KeyValue(1, TrieValue::U32(15)));
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
        let read_result = tokio::time::timeout(Duration::from_secs(1), client.read_base(id1))
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

async fn run_client_test<F, Fut>(runner: F)
where
    F: FnOnce(RemoteClient<TokioSpawnTask>, tokio::sync::mpsc::Receiver<SocketRequest>) -> Fut,
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
                    spawn_local(async move {
                        send_request.send(request).await.expect("sending request");
                    });
                };
                RemoteClient::<TokioSpawnTask>::connect(client_send_socket)
            };
            let client_request_after_connect =
                requests_from_client.recv().await.expect("recv request");
            assert_eq!(client_request_after_connect, SocketRequest::Connect);
            runner(client, requests_from_client).await;
        })
        .await;
}

#[derive(Copy, Clone)]
struct TokioSpawnTask;
impl SpawnTask for TokioSpawnTask {
    fn spawn_task(future: impl Future<Output = ()> + 'static) {
        spawn_local(future);
    }
}
