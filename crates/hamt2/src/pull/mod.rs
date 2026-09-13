use crate::QueryError;
use crate::db::Db;
use crate::trie::prelude::*;
use crate::types::*;
use serde::{Deserialize, Serialize};

pub mod errors;
#[cfg(test)]
mod tests;

pub trait Pull<'a>: Sized + Serialize + Deserialize<'a> {
    fn attrs() -> Vec<Attr>;
    fn into_datoms(self, ent: Ent) -> Vec<Datom>;
    fn pull<S: ReadWriteTrieStorage>(
        db: &Db<S>,
        eid: Ein,
    ) -> impl Future<Output = Result<Self, QueryError>>;
}
