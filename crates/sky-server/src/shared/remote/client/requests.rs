use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{Base, BaseId};
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverStatus(DbStatus),
    RequestBase(BaseId, oneshot::Sender<Option<Base>>),
    DeliverBase(BaseId, Option<Base>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<DbStatus>>),
    DeliverTransact(DbStatus),
    Reconnect,
}
