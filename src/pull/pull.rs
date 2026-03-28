use crate::db::attr::Attr;
use crate::db::Datom;
use crate::db::Ent;
use crate::pull::errors::DatomsError;
use crate::pull::into_datoms;
use serde::{Deserialize, Serialize};

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Result<Vec<Datom>, DatomsError> {
        into_datoms(self, ent)
    }
}
