use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{HandleTrieConfig, SlotBase, SlotBaseId};
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverStatus(DbStatus),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase<HandleTrieConfig>>>),
    DeliverBase(SlotBaseId, Option<SlotBase<HandleTrieConfig>>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<DbStatus>>),
    DeliverTransact(DbStatus),
    Reconnect,
}
