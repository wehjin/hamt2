use crate::db::Datom;
use crate::pull::errors::DatomsError;
use serde::Serialize;
use serial::Serializer;

pub mod db;
pub mod errors;
pub mod pull;
pub mod register;
pub mod serial;

#[cfg(test)]
mod tests;

pub fn into_datoms<S: Serialize>(item: S, eid: i32) -> Result<Vec<Datom>, DatomsError> {
    let mut serializer = Serializer::new(eid);
    let _ = item.serialize(&mut serializer)?;
    let datoms = serializer.datoms;
    Ok(datoms)
}
