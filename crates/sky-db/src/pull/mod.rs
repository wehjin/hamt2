use crate::Db;
use serde::{Deserialize, Serialize};
use crate::Datom;
use crate::QueryError;
use crate::{Attr, Ein, Ent};

pub mod errors;
#[cfg(test)]
mod tests;

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Vec<Datom>;
    fn pull(db: &Db, eid: Ein) -> impl Future<Output = Result<Self, QueryError>>;
}
