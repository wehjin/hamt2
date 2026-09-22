use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::storage::{ReadStorage, ReadStorageError, StorageStatus};
use sky_types::trie::{MapBase, Base, BaseId, RootBaseRead};
use std::marker::PhantomData;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct RemoteClientReadStorage<T: SpawnTask> {
    pub(crate) requester: Sender<ClientRequest>,
    pub(crate) status: std::sync::Arc<std::sync::RwLock<DbStatus>>,
    pub(crate) _spawn_local: PhantomData<T>,
}

impl<T: SpawnTask> RootBaseRead for RemoteClientReadStorage<T> {
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

    fn read_root(&self) -> MapBase {
        // TODO We should wait for the status to arrive instead of return empty and
        // giving the false impression that there is no data.
        self.status.read().unwrap().head.root
    }
}

impl<T: SpawnTask> ReadStorage for RemoteClientReadStorage<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn status(&self) -> StorageStatus {
        self.status.read().unwrap().head
    }

    fn with_new_root(self, new_root: Option<MapBase>) -> Self {
        if let Some(root) = new_root {
            let mut write = self.status.write().unwrap();
            write.head.root = root;
        }
        self
    }

    fn snapshot(&self) -> Self::Snapshot {
        // Deep-clone the status so that future changes in the processing loop do not affect the
        // snapshot.
        let deep_cloned_status = self.status.read().unwrap().clone();
        Self {
            status: std::sync::Arc::new(std::sync::RwLock::new(deep_cloned_status)),
            ..self.clone()
        }
    }
}
impl<T: SpawnTask> RemoteClientReadStorage<T> {
    pub fn to_status(&self) -> DbStatus {
        self.status.read().unwrap().clone()
    }
}
