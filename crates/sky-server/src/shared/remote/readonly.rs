use crate::shared::remote::SpawnTask;
use crate::shared::remote::client::requests::ClientRequest;
use sky_types::db::DbStatus;
use sky_types::storage::view::StoreView;
use sky_types::trie::{Base, Buffer, BufferIndex, CursorPos, MapBase, QueryCursor, TrieSnap};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct Remote<T: SpawnTask> {
    pub(crate) requester: Sender<ClientRequest>,
    pub(crate) status: Arc<RwLock<DbStatus>>,
    pub(crate) _spawn_local: PhantomData<T>,
}

impl<T: SpawnTask> Remote<T> {
    pub fn to_status(&self) -> DbStatus {
        self.status.read().unwrap().clone()
    }
}

impl<T: SpawnTask> QueryCursor for Remote<T> {
    fn cursor_pos(&self) -> &CursorPos {
        unimplemented!()
    }

    fn cursor_pos_mut(&mut self) -> &mut CursorPos {
        unimplemented!()
    }
}

impl<T: SpawnTask> Eq for Remote<T> {}

impl<T: SpawnTask> PartialEq for Remote<T> {
    fn eq(&self, other: &Self) -> bool {
        self.to_status() == other.to_status()
    }
}

impl<T: SpawnTask> StoreView for Remote<T> {}

impl<T: SpawnTask> Buffer for Remote<T> {
    fn max_index(&self) -> BufferIndex {
        self.status.read().unwrap().head.max_id
    }

    fn read_root(&self) -> MapBase {
        // TODO We should wait for the status to arrive instead of return empty and
        // giving the false impression that there is no data.
        self.status.read().unwrap().head.root
    }

    async fn get_base(&self, id: BufferIndex, size: usize) -> Base {
        if id > self.max_index() {
            panic!("invalid base id");
        }
        let (send, recv) = oneshot::channel();
        self.requester
            .send(ClientRequest::RequestBase(id, size, send))
            .await
            .expect("send request");
        let base = recv.await.expect("recv base").expect("base");
        base
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
