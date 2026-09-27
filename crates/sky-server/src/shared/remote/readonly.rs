use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::storage::ReadStorageError;
use sky_types::storage::view::StoreView;
use sky_types::trie::{Base, BaseId, BaseRead, MapBase, TrieSnap, TrieWalk};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct Remote<T: SpawnTask> {
    pub(crate) requester: Sender<ClientRequest>,
    pub(crate) status: Arc<std::sync::RwLock<DbStatus>>,
    pub(crate) _spawn_local: PhantomData<T>,
}

impl<T: SpawnTask> Remote<T> {
    pub fn to_status(&self) -> DbStatus {
        self.status.read().unwrap().clone()
    }
}

impl<T: SpawnTask> StoreView for Remote<T> {}

impl<T: SpawnTask> BaseRead for Remote<T> {
    fn max_id(&self) -> BaseId {
        self.status.read().unwrap().head.max_id
    }

    fn read_root(&self) -> MapBase {
        // TODO We should wait for the status to arrive instead of return empty and
        // giving the false impression that there is no data.
        self.status.read().unwrap().head.root
    }

    async fn read_base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
        if id > self.max_id() {
            panic!("invalid base id");
        }
        let (send, recv) = oneshot::channel();
        self.requester
            .send(ClientRequest::RequestBase(id, send))
            .await
            .expect("send request");
        let base = recv.await.expect("recv base").expect("base");
        Ok(base)
    }
}

impl<T: SpawnTask> TrieSnap for Remote<T> {
    type Snapshot = Self;

    fn snapshot(&self) -> Self::Snapshot {
        // Deep-clone the status so that future changes in the processing loop do not affect the
        // snapshot.
        let status = self.status.read().unwrap().clone();
        Self {
            status: Arc::new(RwLock::new(status)),
            ..self.clone()
        }
    }
}

impl<T: SpawnTask> TrieWalk for Remote<T> {
    type Subtrie = Self;

    fn to_subtrie(&self, subtrie_root: MapBase) -> Self::Subtrie {
        // Deep-clone the status so that future changes in the processing loop do not affect the
        // snapshot.
        let mut status = self.status.read().unwrap().clone();
        status.head.root = subtrie_root;
        Self {
            status: Arc::new(RwLock::new(status)),
            ..self.clone()
        }
    }
}
