use crate::crate_services::datalog::atom::Atom;
use crate::db::Ein;
use crate::db::Schema;
use crate::db::db_trie;
use crate::find::Find;
use crate::trie::prelude::*;
use sky_types::FindResult;
use std::future::Future;

pub struct AllEins;

impl AllEins {
    pub fn new() -> Self {
        Self
    }
}

impl Find for AllEins {
    type Output = Ein;

    fn select(&self) -> Vec<&'static str> {
        unreachable!()
    }

    fn where_(&self) -> Vec<Atom> {
        unreachable!()
    }

    fn process(self, _result: FindResult) -> Vec<Self::Output> {
        unreachable!()
    }

    fn apply<T>(self, trie: &T, _schema: &Schema) -> impl Future<Output = Vec<Self::Output>>
    where
        Self: Sized,
        T: TrieQuery,
    {
        db_trie::list_entities(trie)
    }
}
