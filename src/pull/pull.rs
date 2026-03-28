use crate::db::Datom;
use crate::pull::errors::{BuildError, DatomsError};
use crate::pull::into_datoms;
use serde::Serialize;
use crate::db::attr::Attr;
use crate::db::Val;

pub trait Pull: Sized + Serialize {
    type Target;
    fn attrs() -> Vec<Attr>;
    fn build(bindings: Vec<(Attr, Option<Val>)>) -> Result<Self::Target, BuildError>;
    fn into_datoms(self, id: i32) -> Result<Vec<Datom>, DatomsError> {
        into_datoms(self, id)
    }
}
