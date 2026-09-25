use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::storage::{BaseStore, ReadStorageError, WriteStorageError};
use sky_types::trie::{Base, BaseId, MapBase};
use std::marker::PhantomData;
use std::ops::Deref;
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

impl<T: SpawnTask> BaseStore for Remote<T> {
    fn max_id(&self) -> BaseId {
        self.status.read().unwrap().head.max_id
    }

    fn start_id(&self) -> BaseId {
        BaseId::ZERO
    }

    fn root(&self) -> MapBase {
        // TODO We should wait for the status to arrive instead of return empty and
        // giving the false impression that there is no data.
        self.status.read().unwrap().head.root
    }

    async fn set_root(&mut self, root: MapBase) -> Result<(), WriteStorageError> {
        self.status.write().unwrap().head.root = root;
        Ok(())
    }

    fn with_root(&self, new: Option<MapBase>) -> Self {
        let next_status = self.status.deref().read().unwrap().with_root(new);
        let mut new = self.clone();
        new.status = Arc::new(RwLock::new(next_status));
        new
    }

    async fn base(&self, id: BaseId) -> Result<Base, ReadStorageError> {
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

    async fn push_base(&mut self, _base: Base) -> Result<BaseId, WriteStorageError> {
        unimplemented!()
    }

    async fn extend(&self) -> Result<Self, WriteStorageError>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    async fn commit(self, _past: &Self) -> Result<Self, WriteStorageError>
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn snapshot(&self) -> Self {
        // Deep-clone the status so that future changes in the processing loop do not affect the
        // snapshot.
        let deep_cloned_status = self.status.read().unwrap().clone();
        Self {
            status: Arc::new(RwLock::new(deep_cloned_status)),
            ..self.clone()
        }
    }
}
