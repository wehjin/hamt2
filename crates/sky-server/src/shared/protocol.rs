use serde::{Deserialize, Serialize};
use sky_types::db::{Datom, DbStatus};
use sky_types::trie::{Base, BufferIndex};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketRequest {
    Connect,
    ReadSlotBase(BufferIndex),
    Transact(Vec<Datom>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocketResponse {
    DbStatus(DbStatus),
    SlotBase(BufferIndex, Option<Base>),
    TransactResult(DbStatus),
}