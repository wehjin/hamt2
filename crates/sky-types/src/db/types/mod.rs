mod attr;
mod dat;
mod ein;
mod ent;
mod find_result;
pub mod schema;
mod val;

use crate::db::schema::Schema;
use crate::storage::StorageStatus;
use crate::trie::MapBase;
pub use attr::*;
pub use dat::*;
pub use ein::*;
pub use ent::*;
pub use find_result::*;
use serde::{Deserialize, Serialize};
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
