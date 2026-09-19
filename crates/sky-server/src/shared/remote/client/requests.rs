use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::Datom;
use sky_types::trie::SlotBaseId;
use tokio::sync::oneshot;

pub enum ClientRequest {
    DeliverHead(StorageHead),
    RequestBase(SlotBaseId, oneshot::Sender<Option<SlotBase>>),
    DeliverBase(SlotBaseId, Option<SlotBase>),
    RequestTransact(Vec<Datom>, oneshot::Sender<Option<StorageHead>>),
    DeliverTransact(StorageHead),
    Reconnect,
}
