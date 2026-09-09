use crate::db::{Attr, Datom, Db, Ein, Ent};
use crate::space::Space;
use crate::QueryError;
use serde::{Deserialize, Serialize};

pub mod errors;
#[cfg(test)]
mod tests;

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Vec<Datom>;
    fn pull<T: Space>(db: &Db<T>, eid: Ein) -> impl Future<Output = Result<Self, QueryError>>;
}
