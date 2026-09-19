use sky_trie::types::slot_base::SlotBase;
use sky_types::db::{Datom, DbStatus};
use sky_types::trie::SlotBaseId;
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverStatus(DbStatus),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
    DeliverBase(SlotBaseId, Option<SlotBase>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<DbStatus>>),
    DeliverTransact(DbStatus),
    Reconnect,
}
