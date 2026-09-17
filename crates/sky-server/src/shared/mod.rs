use serde::{Deserialize, Serialize};
use sky_trie::types::StorageHead;
use sky_trie::types::slot_base::SlotBase;
use sky_types::db::Datom;
use sky_types::trie::SlotBaseId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
    ReadSlotBase(SlotBaseId),
    Transact(Vec<Datom>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    StorageStatus(StorageHead),
    SlotBase(SlotBaseId, Option<SlotBase>),
    TransactResult(StorageHead),
}