use crate::db::attr::Attr;
use crate::db::Datom;
use crate::db::{Ent, Val};
use crate::pull::errors::{BuildError, DatomsError};
use crate::pull::into_datoms;
use serde::Serialize;

pub trait Pull: Sized + Serialize {
    type Target;
    fn attrs() -> Vec<Attr>;
    fn build(bindings: Vec<(Attr, Option<Val>)>) -> Result<Self::Target, BuildError>;
    fn into_datoms(self, ent: Ent) -> Result<Vec<Datom>, DatomsError> {
        into_datoms(self, ent)
    }
}
