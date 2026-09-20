use crate::db::Db;
use serde::{Deserialize, Serialize};
use sky_types::db::Datom;
use sky_types::db::QueryError;
use sky_types::db::{Attr, Ein, Ent};
use sky_types::storage::ReadWriteStorage;

pub mod errors;
#[cfg(test)]
mod tests;

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Vec<Datom>;
    fn pull<S: ReadWriteStorage>(
        db: &Db<S>,
        eid: Ein,
    ) -> impl Future<Output = Result<Self, QueryError>>;
}
