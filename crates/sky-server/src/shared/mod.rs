use serde::{Deserialize, Serialize};
use sky_types::db::Datom;
use sky_types::db::DbStatus;
use sky_types::trie::{SlotBase, SlotBaseId};

pub mod remote;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
    ReadSlotBase(SlotBaseId),
    Transact(Vec<Datom>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    DbStatus(DbStatus),
    SlotBase(SlotBaseId, Option<SlotBase>),
    TransactResult(DbStatus),
}
