use crate::db::attr::Attr;
use crate::db::Datom;
use crate::db::Ein;
use crate::db::{Db, Ent};
use crate::pull::errors::DatomsError;
use crate::pull::into_datoms;
use crate::space::Space;
use crate::QueryError;
use serde::{Deserialize, Serialize};

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Result<Vec<Datom>, DatomsError> {
        into_datoms(self, ent)
    }
    fn pull<T: Space>(db: &Db<T>, eid: Ein) -> impl Future<Output = Result<Self, QueryError>>;
}
