use crate::shared::{SocketRequest, SocketResponse};
use sky_trie::storage::ReadStorage;
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::storage::ReadStorageError;
use sky_types::trie::{MapBase, SlotBaseId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub trait SpawnLocal: Clone + Send + Sync + 'static {
    fn spawn_local(future: impl Future<Output = ()> + 'static);
}

enum RemoteStorageRequest {
    UpdateStatus(StorageHead),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
    DeliverBase(SlotBaseId, Option<SlotBase>),
}

#[derive(Clone)]
pub struct RemoteStorage<SpawnLocal> {
    inner: RemoteReadStorage<SpawnLocal>,
}

impl<T: SpawnLocal> RemoteStorage<T> {
    pub fn status(&self) -> &StorageHead {
        &self.inner.head
    }

    fn send_request(&self, request: RemoteStorageRequest) {
        let requester = self.inner.requester.clone();
        T::spawn_local(async move {
            let _ = requester.send(request).await;
        })
    }

    fn update_head(&mut self, head: StorageHead) {
        if head.max_id > self.inner.head.max_id {
            self.inner.head = head;
        }
        self.send_request(RemoteStorageRequest::UpdateStatus(head));
    }

    pub fn update(&mut self, socket_response: SocketResponse) {
        match socket_response {
            SocketResponse::StorageStatus(head) => {
                self.update_head(head);
            }
            SocketResponse::SlotBase(id, base) => {
                self.send_request(RemoteStorageRequest::DeliverBase(id, base));
            }
            SocketResponse::TransactResult(head) => {
                self.update_head(head);
            }
        }
    }

    pub fn connect(send_socket: impl Fn(SocketRequest) + 'static) -> Self {
        let (send_request, mut recv_request) =
            tokio::sync::mpsc::channel::<RemoteStorageRequest>(100);
        let send_socket = Arc::new(send_socket);
        let task_send_socket = send_socket.clone();
        T::spawn_local(async move {
            let mut waiting: HashMap<SlotBaseId, Vec<oneshot::Sender<Option<SlotBase>>>> =
                HashMap::new();
            let mut bases = HashMap::from([(SlotBaseId::ZERO, SlotBase::new())]);
            let mut max_id = SlotBaseId::ZERO;
            loop {
                let event = recv_request.recv().await;
                if let Some(request) = event {
                    match request {
                        RemoteStorageRequest::UpdateStatus(head) => {
                            if head.max_id > max_id {
                                max_id = head.max_id;
                            }
                        }
                        RemoteStorageRequest::RequestBase(id, send_base) => {
                            if id <= max_id && id >= SlotBaseId::ZERO {
                                if let Some(base) = bases.get(&id).cloned() {
                                    let _ = send_base.send(Some(base));
                                } else {
                                    let mut line = waiting.remove(&id).unwrap_or_default();
                                    line.push(send_base);
                                    waiting.insert(id, line);
                                    task_send_socket(SocketRequest::ReadSlotBase(id));
                                }
                            } else {
                                let _ = send_base.send(None);
                            }
                        }
                        RemoteStorageRequest::DeliverBase(id, base) => {
                            let line = waiting.remove(&id).unwrap_or_default();
                            for send_base in line {
                                let base = base.clone();
                                let _ = send_base.send(base);
                            }
                            if let Some(base) = base.clone() {
                                bases.insert(id, base);
                            }
                        }
                    }
                }
            }
        });
        send_socket(SocketRequest::Connect);
        let inner = RemoteReadStorage {
            requester: send_request,
            head: StorageHead {
                max_id: SlotBaseId::ZERO,
                root: MapBase::empty(),
            },
            _spawn_local: PhantomData,
        };
        Self { inner }
    }
}

impl<T: SpawnLocal> ReadStorage for RemoteStorage<T> {
    type Snapshot = RemoteReadStorage<T>;

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

#[derive(Clone)]
pub struct RemoteReadStorage<SpawnLocal> {
    requester: Sender<RemoteStorageRequest>,
    head: StorageHead,
    _spawn_local: PhantomData<SpawnLocal>,
}

impl<T: SpawnLocal> ReadStorage for RemoteReadStorage<T> {
    type Snapshot = RemoteReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
        if id > self.head.max_id {
            panic!("invalid base id");
        }
        let (send, recv) = oneshot::channel();
        self.requester
            .send(RemoteStorageRequest::RequestBase(id, send))
            .await
            .expect("send request");
        let base = recv.await.expect("recv base").expect("base");
        Ok(base)
    }

    fn max_id(&self) -> SlotBaseId {
        self.head.max_id
    }

    fn read_root(&self) -> MapBase {
        self.head.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sky_trie::types::slot::Slot;
    use sky_types::trie::{SlotMap, TrieValue};

    #[tokio::test]
    async fn remote_storage_works() {
        tokio::task::LocalSet::new()
            .run_until(async move {
                let (send_request, mut recv_request) =
                    tokio::sync::mpsc::channel::<SocketRequest>(100);
                let send_request = move |request| {
                    let send_request = send_request.clone();
                    tokio::task::spawn_local(async move {
                        send_request.send(request).await.expect("sending request");
                    });
                };

                let mut remote = RemoteStorage::<TokioSpawnLocal>::connect(send_request);
                let socket_request = recv_request.recv().await.expect("recv request");
                assert!(matches!(socket_request, SocketRequest::Connect));

                let id1 = SlotBaseId(1);
                remote.update(SocketResponse::StorageStatus(StorageHead {
                    max_id: id1,
                    root: MapBase {
                        map: SlotMap::empty(),
                        base: id1,
                    },
                }));
                assert_eq!(remote.status().max_id, id1);

                let task_remote = remote.clone();
                let handle =
                    tokio::task::spawn_local(async move { task_remote.read(id1).await.unwrap() });
                let socket_request = recv_request.recv().await.expect("recv request");
                assert_eq!(SocketRequest::ReadSlotBase(id1), socket_request);

                let base1 = SlotBase::new().insert_slot(0, Slot::KeyValue(1, TrieValue::U32(15)));
                remote.update(SocketResponse::SlotBase(id1, Some(base1.clone())));
                let base = handle.await.expect("recv base");
                assert_eq!(base1, base)
            })
            .await;
    }

    #[derive(Copy, Clone)]
    struct TokioSpawnLocal;
    impl SpawnLocal for TokioSpawnLocal {
        fn spawn_local(future: impl Future<Output = ()> + 'static) {
            tokio::task::spawn_local(future);
        }
    }
}
