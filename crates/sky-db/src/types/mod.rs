mod attr;
mod dat;
mod ein;
mod ent;
mod fill;
mod find_result;
mod schema;
mod val;

pub use crate::_internal::schema::*;
use serde::{Deserialize, Serialize};

mod txid;
pub use attr::*;
pub use dat::*;
pub use ein::*;
pub use ent::*;
pub use fill::*;
pub use find_result::*;
pub use schema::*;
use sky_types::storage::StorageStatus;
use sky_types::trie::MapBase;
pub use txid::*;
pub use val::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Datom {
    pub ent: Ent,
    pub attr: Attr,
    pub dat: Dat,
    pub dir: Dir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DbStatus {
    pub head: StorageStatus,
    pub schema: Schema,
}

impl DbStatus {
    pub fn with_root(&self, root: Option<MapBase>) -> Self {
        let root = root.unwrap_or(self.head.root);
        let Self { head, schema } = self.clone();
        let head = StorageStatus { root, ..head };
        Self { head, schema }
    }
}
