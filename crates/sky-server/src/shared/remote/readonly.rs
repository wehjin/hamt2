use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::ClientRequest;
use sky_trie::prelude::ReadStorage;
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::storage::ReadStorageError;
use sky_types::trie::{MapBase, SlotBaseId};
use std::marker::PhantomData;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct RemoteClientReadStorage<SpawnLocal> {
    pub(crate) requester: Sender<ClientRequest>,
    pub(crate) head: std::sync::Arc<std::sync::RwLock<StorageHead>>,
    pub(crate) _spawn_local: PhantomData<SpawnLocal>,
}

impl<T: SpawnTask> ReadStorage for RemoteClientReadStorage<T> {
    type Snapshot = RemoteClientReadStorage<T>;

    fn snapshot(&self) -> Self::Snapshot {
        // Fix the head of the snapshot in a new Arc so that future changes
        // in the processing loop do not affect it.
        let head = self.get_head();
        let mut clone = self.clone();
        clone.head = std::sync::Arc::new(std::sync::RwLock::new(head));
        clone
    }

    async fn read(&self, id: SlotBaseId) -> Result<SlotBase, ReadStorageError> {
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
        self.head.read().unwrap().max_id
    }

    fn read_root(&self) -> MapBase {
        self.head.read().unwrap().root
    }
}
