use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::storage::{ReadStorage, ReadStorageError};
use sky_types::trie::{
    HandleTrieConfig, MapBase, SlotBase, SlotBaseId, TrieQueryError, TrieBaseRead,
};
use std::marker::PhantomData;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct RemoteClientReadStorage<T: SpawnTask> {
    pub(crate) requester: Sender<ClientRequest>,
    pub(crate) status: std::sync::Arc<std::sync::RwLock<DbStatus>>,
    pub(crate) _spawn_local: PhantomData<T>,
}

impl<T: SpawnTask> TrieBaseRead for RemoteClientReadStorage<T> {
    type Config = HandleTrieConfig;

    async fn read_base(
        &self,
        id: SlotBaseId,
    ) -> Result<SlotBase<HandleTrieConfig>, TrieQueryError> {
        let base = ReadStorage::read(self, id).await?;
        Ok(base)
    }
}

impl<T: SpawnTask> ReadStorage for RemoteClientReadStorage<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        // Fix the status of the snapshot in a new Arc so that future changes
        // in the processing loop do not affect the snapshot.
        let status = self.status.read().unwrap().clone();
        let mut clone = self.clone();
        clone.status = std::sync::Arc::new(std::sync::RwLock::new(status));
        clone
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase<HandleTrieConfig>, ReadStorageError> {
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

    fn max_id(&self) -> SlotBaseId {
        self.status.read().unwrap().head.max_id
    }

    fn read_root(&self) -> MapBase<HandleTrieConfig> {
        self.status.read().unwrap().head.root
    }
}
impl<T: SpawnTask> RemoteClientReadStorage<T> {
    pub fn to_status(&self) -> DbStatus {
        self.status.read().unwrap().clone()
    }
}
