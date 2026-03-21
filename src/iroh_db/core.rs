use crate::iroh_db::reader::Reader;
use crate::QueryError;
use serde::{Deserialize, Serialize};

pub enum Datom {
    Add(Ent, Attr, Val),
    Id(Ent, Vec<(Attr, Val)>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Ent(pub i64);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Attr(pub &'static str);
impl Attr {
    pub async fn query_val(&self, e: Ent, reader: &Reader) -> Result<Option<Val>, QueryError> {
        reader.query_value(e, self).await
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Val {
    Uint(u64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Tx(pub u64);
impl Tx {
    pub fn next(&self) -> Tx {
        Tx(self.0 + 1)
    }
}
