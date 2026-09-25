use serde::{Deserialize, Serialize};
use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{Base, BaseId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
    ReadSlotBase(BaseId),
    Transact(Vec<Datom>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    DbStatus(DbStatus),
    SlotBase(BaseId, Option<Base>),
    TransactResult(DbStatus),
}