use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{Base, BufferIndex};
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverStatus(DbStatus),
    RequestBase(BufferIndex, oneshot::Sender<Option<Base>>),
    DeliverBase(BufferIndex, Option<Base>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<DbStatus>>),
    DeliverTransact(DbStatus),
    Reconnect,
}
