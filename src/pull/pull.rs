use crate::db::{Attr, Datom, Val};
use crate::pull::errors::{BuildError, PushError};
use crate::pull::into_datoms;
use serde::Serialize;

pub trait Pull: Sized + Serialize {
    type Target;
    fn attrs() -> Vec<Attr>;
    fn build(bindings: Vec<(Attr, Option<Val>)>) -> Result<Self::Target, BuildError>;
    fn push(self, id: i32) -> Result<Vec<Datom>, PushError> {
        into_datoms(self, id)
    }
}
