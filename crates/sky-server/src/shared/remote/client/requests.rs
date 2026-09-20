use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{SlotBase, SlotBaseId};
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverStatus(DbStatus),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase<SlotBaseId>>>),
    DeliverBase(SlotBaseId, Option<SlotBase<SlotBaseId>>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<DbStatus>>),
    DeliverTransact(DbStatus),
    Reconnect,
}
